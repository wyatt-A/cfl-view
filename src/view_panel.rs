use array_lib::ArrayDim;
use array_lib::cfl::num_complex::Complex32;
use iced::{Element, Length, Point, Rectangle, Settings, Task, Theme};
use iced::mouse::Cursor;
use iced::widget::{button, canvas, column, container, row, text, Canvas, Image};
use iced::widget::canvas::{Frame, Geometry, Program};
use iced::Renderer;
use iced::widget::image::Handle;

pub struct ViewPanel {

    /// number of panes in the grid
    n_panes:usize,

    /// size of each pane
    pane_dims:[usize;2],

    /// dims of pane grid
    grid_dims:[usize;2],

    view_mode:ViewMode,

    scaling:f32,
}

#[derive(Debug, Clone)]
pub enum ViewPanelMessage {
    Increment
}

impl Default for ViewPanel {

    fn default() -> Self {
        ViewPanel {
            n_panes: 3,
            pane_dims: [512,512],
            grid_dims: [1,3],
            view_mode: ViewMode::default(),
            scaling: 128.0,
        }
    }

}


struct BlankCanvas;

impl ViewPanel {

    pub fn update(&mut self, message: ViewPanelMessage) {

    }

    pub fn view(&self) -> Element<ViewPanelMessage> {
        
        let mut r = column![];

        let mut pane_id = 0;

        for _ in 0..self.grid_dims[0] {
            let mut c = row![];
            for _ in 0..self.grid_dims[1] {
                let (bytes,dims) = self.update_pane(pane_id);
                pane_id += 1;
                c = c.push(
                    container(
                        Image::new(
                            Handle::from_rgba(dims.shape()[0] as u32,dims.shape()[1] as u32,bytes)
                        )
                    ).width(Length::FillPortion(1)).padding(10)
                );
            }
            r = r.push(container(c.spacing(10)).height(Length::FillPortion(1)));
        }
        r.padding(10).into()
    }
}


impl ViewPanel {

    /// returns rgba image bytes for a single pane
    fn update_pane(&self, pane_id:usize) -> (Vec<u8>, ArrayDim) {
        let cfl_data = vec![Complex32::ONE;self.pane_dims[0] * self.pane_dims[1]];
        let bytes = make_rgba(&cfl_data,self.view_mode,self.scaling);
        (bytes,ArrayDim::from_shape(&[self.pane_dims[0],self.pane_dims[1]]))
    }

    fn n_panes(&self) -> usize {
        self.grid_dims[0] * self.grid_dims[1]
    }



}

#[derive(Debug, Clone, Copy)]
enum ViewMode {
    Re,
    Im,
    Mag,
    Phase,
}

impl Default for ViewMode {
    fn default() -> ViewMode {
        ViewMode::Mag
    }
}


fn make_rgba(cfl_data:&[Complex32],view_mode:ViewMode,scaling:f32) -> Vec<u8> {
    let scalars:Vec<f32> = match view_mode {
        ViewMode::Re => cfl_data.iter().map(|x|x.re).collect(),
        ViewMode::Im => cfl_data.iter().map(|x|x.im).collect(),
        ViewMode::Mag => cfl_data.iter().map(|x|x.norm()).collect(),
        ViewMode::Phase => cfl_data.iter().map(|x|x.to_polar().1).collect()
    };
    scalars.into_iter().map(|x| (x * scaling).clamp(u8::MIN as f32,u8::MAX as f32))
        .map(|x| x as u8)
        .map(|x|[x,x,x,u8::MAX])
        .flatten()
        .collect()
}


// impl<Message> Program<Message> for BlankCanvas {
//     type State = ();
//
//     fn draw(
//         &self,
//         _state: &Self::State,
//         renderer: &Renderer,
//         _theme: &Theme,
//         bounds: Rectangle,
//         _curser:Cursor,
//     ) -> Vec<Geometry> {
//
//         let mut frame = Frame::new(renderer, bounds.size());
//
//         // Border so we see the canvas
//         let rect = canvas::Path::rectangle(Point::ORIGIN, bounds.size());
//         frame.stroke(&rect, canvas::Stroke::default());
//
//         vec![frame.into_geometry()]
//     }
// }