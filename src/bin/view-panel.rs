use iced;
use cfl_view::view_panel::ViewPanel;

fn main() -> iced::Result {

    iced::run(ViewPanel::update,ViewPanel::view)

}