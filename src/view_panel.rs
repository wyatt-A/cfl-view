use iced::{Element, Length, Point, Rectangle, Settings, Task, Theme};
use iced::mouse::Cursor;
use iced::widget::{button, canvas, column, container, row, text, Canvas};
use iced::widget::canvas::{Frame, Geometry, Program};
use iced::Renderer;

pub struct ViewPanel {

    /// number of panes in the grid
    n_panes:usize,

    /// size of each pane
    pane_dims:[usize;2],

    /// dims of pane grid
    grid_dims:[usize;2],

}

#[derive(Debug, Clone)]
pub enum ViewPanelMessage {
    Increment
}

impl Default for ViewPanel {

    fn default() -> Self {
        ViewPanel {
            n_panes: 3,
            pane_dims: [128,128],
            grid_dims: [1,3],
        }
    }

}


struct BlankCanvas;

impl ViewPanel {

    pub fn update(&mut self, message: ViewPanelMessage) {

    }

    pub fn view(&self) -> Element<ViewPanelMessage> {

        let canvas = || {
            Canvas::new(BlankCanvas)
                .width(Length::Fill)
                .height(Length::Fill)
        };

        row![
        container(canvas()).width(Length::FillPortion(1)),
        container(canvas()).width(Length::FillPortion(1)),
        container(canvas()).width(Length::FillPortion(1)),
    ]
            .spacing(10)
            .into()

    }
}

impl<Message> Program<Message> for BlankCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _curser:Cursor,
    ) -> Vec<Geometry> {

        let mut frame = Frame::new(renderer, bounds.size());

        // Border so we see the canvas
        let rect = canvas::Path::rectangle(Point::ORIGIN, bounds.size());
        frame.stroke(&rect, canvas::Stroke::default());

        vec![frame.into_geometry()]
    }
}