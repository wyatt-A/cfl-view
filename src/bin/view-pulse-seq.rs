use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use iced;
use iced::{Element, Length, Point, Task, Theme};
use iced::widget::{button, text, column, container, tooltip};
use iced::widget::tooltip::Position;
use rfd::FileDialog;
use iced_aksel;
use iced_aksel::{axis, Axis, Chart, Measure, Plot, PlotData, PlotPoint, Stroke};
use iced_aksel::plot::DragDelta;
use iced_aksel::scale::Linear;
use iced_aksel::shape::{Ellipse, Line};

const X_ID: &str = "time";
const GRAD_ID: &str = "grad";


struct State {
    pulse_seq_file:Option<PathBuf>,
    sample_buffer: Vec<f64>,
    chart_state:iced_aksel::State<&'static str,f64>,
    plot_data_gx: Scatter,
    hover_text: Option<String>,
    plot_bounds_t:[f64; 2],
    plot_bounds_y:[f64; 2],
    zoom_box:Option<[[f64;2];2]>,
}

#[derive(Clone,Debug)]
enum Message {
    PickFileClicked,
    FilePicked(Option<PathBuf>),
    LoadFileClicked,
    PlotHover(Point),
    ChartDrag(DragDelta),
    ChartClicked(Point),
}

impl Default for State {
    fn default() -> Self {

        let mut chart_state = iced_aksel::State::new();


        chart_state.set_axis(
            X_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom),
        );

        chart_state.set_axis(
            GRAD_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Left),
        );

        Self {
            pulse_seq_file: Some(PathBuf::from("/Users/Wyatt/seq-lib/test_out/dti_fse.pshdr")),
            sample_buffer: Vec::new(),
            plot_data_gx: Scatter::new(),
            chart_state,
            hover_text: None,
            plot_bounds_t: [0.,1.],
            plot_bounds_y: [-1.,1.],
            zoom_box: None,
        }
    }
}


fn main() -> iced::Result {

    //iced::run(update,view)

    iced::application(State::default,update,view).theme(iced::theme::Theme::CatppuccinMocha).run()
}


fn update(state:&mut State,message:Message) -> Task<Message> {

    match message {
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
                    println!("loaded {} samples from disk",state.sample_buffer.len());
                    state.plot_data_gx = Scatter::from_buffer(&state.sample_buffer,7,0,1);

                    state.plot_bounds_t[0] = state.plot_data_gx.points.first().unwrap().x;
                    state.plot_bounds_t[1] = state.plot_data_gx.points.last().unwrap().x;

                    state.chart_state.set_axis(X_ID,Axis::new(Linear::new(state.plot_bounds_t[0], state.plot_bounds_t[1]), axis::Position::Bottom));
                    state.chart_state.set_axis(GRAD_ID,Axis::new(Linear::new(state.plot_bounds_y[0], state.plot_bounds_y[1]), axis::Position::Left));

                }
            }
            Task::none()
        }
        Message::PlotHover(point) => {

            // transform point to plot coordinates
            let t = (state.plot_bounds_t[1] - state.plot_bounds_t[0]) * point.x as f64 + state.plot_bounds_t[0];
            let y = (state.plot_bounds_y[1] - state.plot_bounds_y[0]) * point.y as f64 + state.plot_bounds_y[0];

            let t = t * 0.1e-3;

            state.hover_text = Some(format!("time_ms: {t}\ngrad_tpm: {y}"));
            Task::none()
        }
        Message::ChartDrag(delta) => {
            println!("drag:{:?}",delta);
            state.zoom_box.as_mut().map(|zoom|{
                zoom[1][0] = zoom[0][0] + delta.x as f64;
                zoom[1][1] = zoom[0][1] + delta.y as f64;
            });
            println!("zoom :{:#?}",state.zoom_box);
            Task::none()
        }
        Message::ChartClicked(point) => {
            println!("click:{:?}",point);
            state.zoom_box = Some([
                [point.x as f64,point.y as f64],
                [0.,0.]
            ]);
            Task::none()
        }
    }


}

fn view(state:&State) -> Element<Message> {

    let chart = Chart::new(&state.chart_state)
        .plot_data(&state.plot_data_gx, X_ID, GRAD_ID)
        .
        .on_hover(|point| Message::PlotHover(point))
        .on_click(|point|Message::ChartClicked(point))
        .on_
        .on_drag(|drag_delta| Message::ChartDrag(drag_delta));
        //.on_axis_hover(|axis_id, value| Message::PlotHover(axis_id, value));

    let tip = if let Some(dp) = &state.hover_text {
        text(dp.clone())
    } else {
        text("Hover chart")
    };

    column![
        button("choose ps file").on_press(Message::PickFileClicked),
        text(format!("file: {}",state.pulse_seq_file.as_ref().map(|x|x.to_str().unwrap()).unwrap_or("None"))),
        button("load").on_press(Message::LoadFileClicked),
        tooltip(
            container(chart).width(Length::Fill).height(Length::Fill).padding(10),
            container(tip).padding(8),
            Position::FollowCursor,
        )
    ].into()
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

#[derive(Debug)]
struct Scatter {
    points: Vec<PlotPoint<f64>>,
}

impl Scatter {
    pub fn new() -> Self {
        Self{points: Vec::new()}
    }

    /// gathers plot points from a larger buffer of points in memory with multiple y-axes
    pub fn from_buffer(buffer:&[f64],stride:usize,x_offset:usize,y_offset:usize)-> Scatter {
        Scatter {
            points: buffer.chunks(stride).map(|chunk| PlotPoint::new(chunk[x_offset],chunk[y_offset]) ).collect()
        }
    }

}

impl Default for Scatter {
    fn default() -> Self {
        Self {
            points: vec![
                PlotPoint::new(10.0, 20.0),
                PlotPoint::new(50.0, 80.0),
                PlotPoint::new(90.0, 30.0),
            ],
        }
    }
}

impl PlotData<f64> for Scatter {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {


        for seg in self.points.windows(2) {
            plot.add_shape(
                Line::new(seg[0], seg[1]).stroke(Stroke::new(theme.palette().primary,Measure::Screen(1.)))
            )
        }

        // for p in &self.points {
        //     plot.add_shape(
        //         Ellipse::new(*p, Measure::Screen(4.0), Measure::Screen(4.0))
        //             .fill(theme.palette().primary),
        //     );
        // }
    }
}