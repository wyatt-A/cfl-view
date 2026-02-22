use std::cmp::Ordering;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use bytemuck::PodCastError::TargetAlignmentGreaterAndInputNotAligned;
use iced;
use iced::{keyboard, Color, Element, Length, Point, Subscription, Task, Theme};
use iced::keyboard::Modifiers;
use iced::mouse::ScrollDelta;
use iced::widget::{button, text, column, container, tooltip, row, toggler};
use iced::widget::tooltip::Position;
use rfd::FileDialog;
use iced_aksel;
use iced_aksel::{axis, Axis, Chart, Measure, Plot, PlotData, PlotPoint, Stroke};
use iced_aksel::axis::{TickContext, TickResult};
use iced_aksel::plot::DragDelta;
use iced_aksel::scale::Linear;
use iced_aksel::shape::{Ellipse, Line};

// axis IDs
const T_ID: &str = "time";
const GRAD_ID: &str = "grad";
const RF_ID: &str = "rf";

struct State {
    pulse_seq_file:Option<PathBuf>,
    sample_buffer: Vec<f64>,
    chart_state:iced_aksel::State<&'static str,f64>,

    grad_line_series: [LineSeries;3],
    rf_line_series: [LineSeries;2],

    hover_text: Option<String>,

    default_plot_bounds_t:[f64; 2],
    default_plot_bounds_grad:[f64; 2],
    default_plot_bounds_rf:[f64; 2],

    modifiers: Modifiers,

    grad_x_visible:bool,
    grad_y_visible:bool,
    grad_z_visible:bool,
    rf_visible:bool,
}

#[derive(Clone,Debug)]
enum Message {
    PickFileClicked,
    FilePicked(Option<PathBuf>),
    LoadFileClicked,
    PlotHover(Point),
    ChartDrag(DragDelta),
    ChartScroll(Point,ScrollDelta),
    ChartClicked(Point),
    ModifiersChanged(Modifiers),
    ResetView,
    ViewToggle(bool, Channel)
}

#[derive(Clone,Debug)]
enum Channel {
    GX,
    GY,
    GZ,
    RfMag,
    RfRe,
    RfIm,
    RfPhase,
    Acq,
}

impl Default for State {
    fn default() -> Self {

        let mut chart_state = iced_aksel::State::new();


        chart_state.set_axis(
            T_ID,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom)
                .with_tick_renderer(t_axis_tick_renderer)
        );


        chart_state.set_axis(
            GRAD_ID,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Left)
                .with_tick_renderer(t_axis_tick_renderer)
        );

        chart_state.set_axis(
            RF_ID,
            Axis::new(Linear::new(0.0, 10.0), axis::Position::Right)
                .with_tick_renderer(t_axis_tick_renderer)
        );

        let ls = LineSeries::new();

        Self {
            pulse_seq_file: Some(PathBuf::from("dti_fse.pshdr")),
            sample_buffer: Vec::new(),
            grad_line_series: [ls.clone(),ls.clone(),ls.clone()],
            rf_line_series: [ls.clone(),ls.clone()],
            chart_state,
            hover_text: None,
            default_plot_bounds_t: [0.,1.],
            default_plot_bounds_grad: [-1.,1.],
            default_plot_bounds_rf: [0.,1.],
            modifiers: Modifiers::default(),
            grad_x_visible: true,
            grad_y_visible: true,
            grad_z_visible: true,
            rf_visible: true,
        }
    }
}


fn main() -> iced::Result {
    iced::application(State::default,update,view)
        .subscription(subscription)
        .theme(Theme::CatppuccinMocha)
        .run()
}

fn apply_chart_bounds(state:&mut State) {
    state.chart_state.set_domain(&T_ID,state.default_plot_bounds_t[0],state.default_plot_bounds_t[1]);
    state.chart_state.set_domain(&GRAD_ID,state.default_plot_bounds_grad[0],state.default_plot_bounds_grad[1]);
    state.chart_state.set_domain(&RF_ID,state.default_plot_bounds_grad[0],state.default_plot_bounds_grad[1]);
}

fn update(state:&mut State,message:Message) -> Task<Message> {

    match message {
        Message::ModifiersChanged(modifiers) => {
            state.modifiers = modifiers;
            Task::none()
        }
        Message::PickFileClicked => {
            let starting_dir = state.pulse_seq_file.as_ref().map(|x|x.parent().unwrap().to_path_buf());
            Task::perform(pick_file(starting_dir), Message::FilePicked)
        } ,
        Message::FilePicked(path) => {
            if let Some(path) = path {
                state.pulse_seq_file = Some(path);
                Task::none()
            }else {
                Task::none()
            }
        }
        Message::LoadFileClicked => {
            if let Some(path) = state.pulse_seq_file.as_ref() {
                if let Ok(mut f) = File::open(path.with_extension("ps")) {
                    let mut contents = vec![];
                    f.read_to_end(&mut contents).unwrap();
                    let samples:Vec<f64> = bytemuck::cast_slice(&contents).to_vec();
                    state.sample_buffer = samples;

                    let t_min = state.sample_buffer.chunks_exact(7).map(|chunk| chunk[0]).min_by(|a,b|a.partial_cmp(b).unwrap()).unwrap().to_owned();
                    let t_max = state.sample_buffer.chunks_exact(7).map(|chunk| chunk[0]).max_by(|a,b|a.partial_cmp(b).unwrap()).unwrap().to_owned();

                    state.default_plot_bounds_t = [t_min,t_max];

                    println!("loaded {} samples from disk",state.sample_buffer.len());
                    state.grad_line_series = [
                        LineSeries::from_buffer(&state.sample_buffer, 7, 0, 1, Color::from_rgba(0.,1.,0.,1.)),
                        LineSeries::from_buffer(&state.sample_buffer, 7, 0, 2, Color::from_rgba(0.,0.,1.,1.)),
                        LineSeries::from_buffer(&state.sample_buffer, 7, 0, 3, Color::from_rgba(1.,0.,0.,1.)),
                    ];

                    state.rf_line_series = [
                        LineSeries::from_buffer(&state.sample_buffer,7,0,4, Color::from_rgba(1.,0.,0.,1.)),
                        LineSeries::from_buffer(&state.sample_buffer,7,0,5, Color::from_rgba(0.,0.,1.,1.)),
                    ];

                    // find the plot bounds for the gradients
                    state.default_plot_bounds_grad[0] = state.grad_line_series.iter().map(|data|{
                        data.points.iter().min_by(|a,b|a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap()
                    }).min_by(|a,b| a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap().y;

                    state.default_plot_bounds_grad[1] = state.grad_line_series.iter().map(|data|{
                        data.points.iter().max_by(|a,b|a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap()
                    }).max_by(|a,b| a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap().y;

                    // state.default_plot_bounds_rf[0] = state.rf_line_series.iter().map(|data|{
                    //     data.points.iter().min_by(|a,b|a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap()
                    // }).min_by(|a,b| a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap().y;
                    //
                    // state.default_plot_bounds_rf[1] = state.rf_line_series.iter().map(|data|{
                    //     data.points.iter().max_by(|a,b|a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap()
                    // }).max_by(|a,b| a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)).unwrap().y;

                    apply_chart_bounds(state);
                }
            }
            Task::none()
        }
        Message::PlotHover(point) => {

            // transform point to plot coordinates
            let t = (state.default_plot_bounds_t[1] - state.default_plot_bounds_t[0]) * point.x as f64 + state.default_plot_bounds_t[0];
            let y = (state.default_plot_bounds_grad[1] - state.default_plot_bounds_grad[0]) * point.y as f64 + state.default_plot_bounds_grad[0];

            let t = t * 0.1e-3;

            state.hover_text = Some(format!("time_ms: {t}\ngrad_tpm: {y}"));
            Task::none()
        }
        Message::ChartDrag(delta) => {
            state.chart_state.pan_axes(T_ID, GRAD_ID, delta.x, delta.y);
            state.chart_state.pan_axes(T_ID, RF_ID, delta.x, delta.y);
            Task::none()
        }
        Message::ChartScroll(cursor_norm,scroll_delta) => {
            let delta_y = match scroll_delta {
                ScrollDelta::Lines { y, x, .. }
                | ScrollDelta::Pixels { y, x, .. } => {
                    // on mac, holding shift changes the scroll axis. We'll just grab the non-zero delta
                    if y == 0.0 {
                        x
                    }else {
                        y
                    }
                },
            };

            let factor = if delta_y > 0.0 { 1.10 } else { 0.90 };
            // 2. Determine Targets (Default: X, Shift: Y, Ctrl: Both)
            let zoom_x = state.modifiers.command() || !state.modifiers.shift();
            let zoom_y = state.modifiers.command() || state.modifiers.shift();

            // 3. Apply Zoom
            if zoom_x && let Some(axis) = state.chart_state.axis_mut_opt(&T_ID) {
                axis.zoom(factor, Some(cursor_norm.x));
            }
            if zoom_y && let Some(axis) = state.chart_state.axis_mut_opt(&GRAD_ID) {
                axis.zoom(factor, Some(cursor_norm.y));
            }
            if zoom_y && let Some(axis) = state.chart_state.axis_mut_opt(&RF_ID) {
                axis.zoom(factor, Some(cursor_norm.y));
            }

            Task::none()
        },
        Message::ChartClicked(point) => {
            Task::none()
        },
        Message::ResetView => {
            apply_chart_bounds(state);
            Task::none()
        },
        Message::ViewToggle(visible, channel) => {
            match channel {
                Channel::GX => state.grad_x_visible = visible,
                Channel::GY => state.grad_y_visible = visible,
                Channel::GZ => state.grad_z_visible = visible,
                Channel::RfRe => state.rf_visible = visible,
                _=> {}
            }
            state.grad_line_series[0].set_visibility(state.grad_x_visible);
            state.grad_line_series[1].set_visibility(state.grad_y_visible);
            state.grad_line_series[2].set_visibility(state.grad_z_visible);
            state.rf_line_series[0].set_visibility(state.rf_visible);
            state.rf_line_series[1].set_visibility(state.rf_visible);
            Task::none()
        }
    }


}

fn view(state:&State) -> Element<Message> {

    let chart = Chart::new(&state.chart_state)
        .plot_data(&state.grad_line_series[0], T_ID, GRAD_ID)
        .plot_data(&state.grad_line_series[1], T_ID, GRAD_ID)
        .plot_data(&state.grad_line_series[2], T_ID, GRAD_ID)
        .plot_data(&state.rf_line_series[0], T_ID, RF_ID)
        .plot_data(&state.rf_line_series[1], T_ID, RF_ID)
        .on_hover(|point| Message::PlotHover(point))
        .on_scroll(|cursor_norm,scroll_delta| Message::ChartScroll(cursor_norm,scroll_delta))
        .on_click(|point|Message::ChartClicked(point))
        .on_drag(|drag_delta| Message::ChartDrag(drag_delta));

    let tip = if let Some(dp) = &state.hover_text {
        text(dp.clone())
    } else {
        text("Hover chart")
    };


    let controls = column![
        button("choose ps file").on_press(Message::PickFileClicked),
        text(format!("file: {}",state.pulse_seq_file.as_ref().map(|x|x.to_str().unwrap()).unwrap_or("None"))),
        button("load").on_press(Message::LoadFileClicked),
        button("reset view").on_press(Message::ResetView),
        text("Visibility"),
        toggler(state.grad_x_visible).label("grad-x").on_toggle(|state| Message::ViewToggle(state,Channel::GX)),
        toggler(state.grad_y_visible).label("grad-y").on_toggle(|state| Message::ViewToggle(state,Channel::GY)),
        toggler(state.grad_z_visible).label("grad-z").on_toggle(|state| Message::ViewToggle(state,Channel::GZ)),
        toggler(state.rf_visible).label("rf").on_toggle(|state| Message::ViewToggle(state,Channel::RfRe)),
    ].spacing(10).padding(10);


    let plot = tooltip(
            container(chart).width(Length::Fill).height(Length::Fill).padding(10),
            container(tip).padding(8),
            Position::FollowCursor,
    );

    row![controls, plot].into()

}

pub fn subscription(_state:&State) -> Subscription<Message> {
    // Listen for modifier keys to enable axis-locking
    iced::event::listen_with(|event, _status, _window_id| {
        if let iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            Some(Message::ModifiersChanged(modifiers))
        } else {
            None
        }
    })
}



async fn pick_file(starting_dir:Option<PathBuf>) -> Option<PathBuf> {
    file_dialog(starting_dir)
}

fn file_dialog(starting_directory:Option<PathBuf>) -> Option<PathBuf> {


    let start_dir = if let Some(starting_directory) = starting_directory {
        Some(starting_directory)
    }else {

        if let Ok(current_dir) = std::env::current_dir() {
            Some(current_dir)
        }else {
            None
        }
    };

    if let Some(start_dir) = start_dir {
        FileDialog::new()
            .add_filter("pshdr files", &["pshdr", "ps"])
            .set_directory(start_dir)
            .pick_file()
    }else {
        FileDialog::new()
            .add_filter("pshdr files", &["pshdr", "ps"])
            .pick_file()
    }


}

#[derive(Debug,Clone)]
struct LineSeries {
    visible: bool,
    points: Vec<PlotPoint<f64>>,
    color: Color,
}

impl LineSeries {
    pub fn new() -> Self {
        Self{points: Vec::new(),color:Color::BLACK, visible:true}
    }

    /// gathers plot points from a larger buffer of points in memory with multiple y-axes
    pub fn from_buffer(buffer:&[f64],stride:usize,x_offset:usize,y_offset:usize, color:Color)-> LineSeries {
        LineSeries {
            visible: true,
            points: buffer.chunks(stride).map(|chunk| PlotPoint::new(chunk[x_offset], chunk[y_offset]) ).collect(),
            color,
        }
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.visible = visible;
    }

}

impl Default for LineSeries {
    fn default() -> Self {
        Self::new()
    }
}

impl PlotData<f64> for LineSeries {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        if self.visible {
            for seg in self.points.windows(2) {
                plot.add_shape(
                    Line::new(seg[0], seg[1]).stroke(Stroke::new(self.color,Measure::Screen(1.)))
                )
            }
        }
    }
}

fn t_axis_tick_renderer(ctx: TickContext<f64, Theme>) -> TickResult {
    if ctx.tick.level != 0 {
        return TickResult::default();
    }

    TickResult {
        label: Some(ctx.label(format!("{:.0}", ctx.tick.value))),
        tick_line: Some(ctx.tickline()),
        ..Default::default()
    }
}