use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use iced;
use iced::{Element, Task};
use iced::widget::{button, text, column};
use rfd::FileDialog;
use iced_aksel;
use iced_aksel::Chart;


const X_ID: &str = "time";
const GRAD_ID: &str = "grad";


struct State {
    pulse_seq_file:Option<PathBuf>,
    sample_buffer: Vec<f64>,
    chart_state:iced_aksel::State<&'static str,f64>,
}

#[derive(Clone,Debug)]
enum Message {
    PickFileClicked,
    FilePicked(Option<PathBuf>),
    LoadFileClicked,
}

impl Default for State {
    fn default() -> Self {
        Self {
            pulse_seq_file: None,
            sample_buffer: Vec::new(),
            chart_state: iced_aksel::State::new(),
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
                }
            }
            Task::none()
        }
    }


}

fn view(state:&State) -> Element<Message> {

    let chart = Chart::new(&state.chart_state)
        .plot_data(&app.series, X_ID, GRAD_ID);

    column![
        button("choose ps file").on_press(Message::PickFileClicked),
        text(format!("file: {}",state.pulse_seq_file.as_ref().map(|x|x.to_str().unwrap()).unwrap_or("None"))),
        button("load").on_press(Message::LoadFileClicked),

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