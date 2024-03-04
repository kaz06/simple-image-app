
use std::fmt;
use std::collections::HashMap;
use std::error::Error;
use iced::widget::{column, container, pick_list, text_input, Button, Column, Container, Image, Row, Text};
use iced::{alignment, executor, Alignment, Application, Command, Element, Length, Renderer, Settings, Theme};
use process::{image_load, process_image, ImagePanelData, ImageProcessError, ImageType, ProcessType};
use rfd::FileDialog;

const LOAD_FILE_EXTENTIONS: &[&str; 7]  = &["jpg", "jpeg", "png", "bmp", "gif", "tiff", "tif"];

mod process;
fn main() {

    let mut setting: Settings<()> = Settings::default();
    setting.window.size = (1200, 900);
    let _ = ImageProcessSample::run(setting);
}

#[derive(Debug, Clone)]
enum Message {
    ProcessTypeSelected(ProcessType),
    PathChanged(String),
    ImageLoad,
    Process,
    ShowFileDialog,
    ProcessEnd(Result<ImagePanelData, ImageProcessError>),
}

struct  UserInteractItems {
    process_type: ProcessType,
    path: String,
    is_image_loaded: bool,
}

enum PanelInfoImageValueState {
    Set(usize),
    Unset,
}

impl fmt::Display for PanelInfoImageValueState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PanelInfoImageValueState::Set(value) => write!(f, "{}", value),
            PanelInfoImageValueState::Unset => write!(f, ""),
        }
    }
}
struct PanelInformation {
    image_load_result: String,
    image_process_result: String,
    processed_type: ProcessType,
    image_width: PanelInfoImageValueState,
    image_height: PanelInfoImageValueState,
    max_image_value: PanelInfoImageValueState,
    min_image_value: PanelInfoImageValueState,
}   
#[derive(Debug, Clone)]
struct ImagePanel {
    images: HashMap<ImageType, ImagePanelData>,
}

struct ImageProcessSample {
    user_interact_items: UserInteractItems,
    panel_information: PanelInformation,
    image_panel: ImagePanel,
    is_processing: bool,
}

trait RowContent {
    fn new() -> Self;
    fn to_row(&self) -> Row<'_, Message>;
}

impl RowContent for UserInteractItems {
    fn new() -> Self {
        UserInteractItems {
            process_type: ProcessType::None,
            path: String::from(""),
            is_image_loaded: false,
        }
    }
    fn to_row(&self) -> Row<'static, Message> {
        let pick_list = pick_list(
            ProcessType::ALL,
            Some(self.process_type),
            Message::ProcessTypeSelected,
        );

        let load_button = 
        Button::new(Text::new("Load"),
        )
        .on_press(Message::ImageLoad);



        let process_button = Button::new(
            Text::new("Process"),
        )
        .on_press(Message::Process);

        let file_dialog  = Button::new(
            Text::new("Select"),
        )
        .on_press(Message::ShowFileDialog)
        ;

        let text_input = 
        text_input("image file path", &self.path,)
        .width(Length::Fill)
        .on_input( Message::PathChanged);

        let process_row = Row::new()
            .spacing(10)
            .align_items(Alignment::Start)
            .push(pick_list)
            .push(process_button);

        let file_path_row = Row::new()
            .spacing(10)
            .align_items(Alignment::Start)
            .push(file_dialog)
            .push(load_button)
            .push(text_input);

        let user_interact_items_row = Row::new()
            .align_items(Alignment::Start)
            .push(        column![
                column![file_path_row].padding(10),
                column![process_row].padding(10),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(50);

        user_interact_items_row
    }
}


impl RowContent for PanelInformation {
    fn new() -> Self {
        PanelInformation {
            image_load_result: String::from(""),
            image_process_result: String::from(""),
            processed_type: ProcessType::None,
            image_width: PanelInfoImageValueState::Unset,
            image_height: PanelInfoImageValueState::Unset,
            max_image_value: PanelInfoImageValueState::Unset,
            min_image_value: PanelInfoImageValueState::Unset,
        }
    }
    fn to_row(&self) -> Row<'static, Message> {
        fn create_row(caption: &str, value: &str) -> Row<'static, Message> {
            let caption = Text::new(String::from(caption));
            let value = Text::new(String::from(value));
            let row = Row::new()
                .spacing(0)
                .push(caption)
                .push(value);
            row
        }
        let iamge_load_result_row = create_row("Image load result: ", &self.image_load_result);
        let image_process_result_row = create_row("Image process result: ", &self.image_process_result);
        let processed_type_row = create_row("Processed type: ", &self.processed_type.to_string());
        let image_width_row = create_row("Image width: ", &self.image_width.to_string());
        let image_height_row = create_row("Image height: ", &self.image_height.to_string());
        let max_image_value_row = create_row("Maximum pixel value (16-bit): ", &self.max_image_value.to_string());
        let min_image_value_row = create_row("Minimum pixel value (16-bit): ", &self.min_image_value.to_string());
    
        let panel_information_row = Row::new()
            .align_items(alignment::Alignment::Start)
            .push(
                column![
                    column![iamge_load_result_row].padding(3),
                    column![image_process_result_row].padding(3),
                    column![processed_type_row].padding(3),
                    column![image_width_row].padding(3),
                    column![image_height_row].padding(3),
                    column![max_image_value_row].padding(3),
                    column![min_image_value_row].padding(3),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill).padding(50);
    
        panel_information_row
    }

}

impl RowContent for ImagePanel {
    fn new() -> Self {
        let images = HashMap::new();
        ImagePanel {
            images,
        }
    }
    fn to_row(&self) -> Row<'static, Message> {
        let row = Row::new()
            .spacing(0)
            .align_items(Alignment::Center)
            .push(self.create_image_panel(ImageType::Original))
            .push(self.create_image_panel(ImageType::Grayscale))
            .push(self.create_image_panel(ImageType::Processed));
        row.padding(50)
    }}

impl ImagePanel {

    fn to_content(&self, image_type: ImageType) -> Element<'static, Message, Renderer> {
        if !self.images.contains_key(&image_type) {
            return Container::new(Text::new("No Image")).into();
        }
        match image_type {
            ImageType::Original => Image::new(self.images[&image_type].to_rgba8_image_handle().clone()).into(),
            _=> Image::new(self.images[&image_type].to_handle().clone()).into(),
        }
    }
    fn create_image_panel(&self, image_type: ImageType) -> Container<'static, Message> {
        let grid_cell_style = |_: &iced::Theme| container::Appearance {
            text_color: None,
            background: None,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.7, 0.7, 0.7),
            border_radius: 0.0.into(),
        };
        let row_height = 300.0; 
        container(self.to_content(image_type))
                    .width(Length::Fill)
                    .height(Length::Fixed(row_height))
                    .center_x()
                    .center_y()
                    .style(grid_cell_style)
    }

    fn image_load(&mut self, path: &str) ->  Result<String, Box<dyn Error>>{
         image_load(& mut self.images, path) 
    }


}


impl ImageProcessSample {
    fn new() -> Self {
        ImageProcessSample {
            user_interact_items: UserInteractItems::new(),
            panel_information: PanelInformation::new(),
            image_panel: ImagePanel::new(),
            is_processing: false,
        }
    }


    fn image_load(&mut self) {
        match self.image_panel.image_load(&self.user_interact_items.path) {
            Ok(message) => {
                self.user_interact_items.is_image_loaded = true;
                self.panel_information.image_load_result = message;
            },
            Err(e) => {
                self.user_interact_items.is_image_loaded = false;
                self.panel_information.image_load_result = e.to_string();
                self.user_interact_items.process_type = ProcessType::None;
                self.panel_information.processed_type = ProcessType::None;
                self.panel_information.image_process_result = String::from("");
                self.statics_reset();
                self.image_panel.images.clear();
            }
        
        };
    }

    fn file_path_select(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Image", LOAD_FILE_EXTENTIONS)
            .pick_file()
        {
            let path_str =path.as_path().to_str();
            match path_str {
                Some(path) => {
                    self.user_interact_items.path = path.to_string();
                },
                None => {
                    self.user_interact_items.path = String::from("");
                }
            }

        }
    }
    fn statics_reset(&mut self) {
        self.panel_information.image_width = PanelInfoImageValueState::Unset;
        self.panel_information.image_height = PanelInfoImageValueState::Unset;
        self.panel_information.max_image_value = PanelInfoImageValueState::Unset;
        self.panel_information.min_image_value = PanelInfoImageValueState::Unset;
    }
}

impl Application for ImageProcessSample {
    type Message = Message;
    type Executor  = executor::Default;
    type Theme = Theme;
    type Flags = ();
    
    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
            (
                ImageProcessSample::new(), Command::none()
            )
    }
    fn title(&self) -> String {
        String::from("Image Process Sample")
    }
    fn view(&self) -> iced::Element<'_, Self::Message> {
        
        let grid = Column::new().spacing(0).align_items(Alignment::Center)
            .push(self.user_interact_items.to_row())
            .push(self.panel_information.to_row())
            .push(self.image_panel.to_row());

        let content = container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();
        if self.is_processing{
            let processing_text = Container::new(Text::new("Processing...").size(50))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y();
            
                processing_text.into()
        } else {
            content.into()
        
        }
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {

        match message {
            Message::ProcessTypeSelected(process_type) => {
                self.user_interact_items.process_type = process_type;
                Command::none()
            },
            Message::PathChanged(path) => {self.user_interact_items.path = path; Command::none()},
            Message::ImageLoad => {self.image_load();  Command::none()},
            Message::ShowFileDialog => {self.file_path_select(); Command::none()},
            Message::Process => {
                if self.user_interact_items.is_image_loaded {
                    self.is_processing = true;
                    Command::perform(process_image(
                        self.image_panel.images.get(&ImageType::Grayscale).unwrap().clone(), 
                        self.user_interact_items.process_type), Message::ProcessEnd)
                } else {
                    Command::none()
                }

            },
            Message::ProcessEnd(result) => {
                self.is_processing = false;
                match result {
                    Ok(image_panel_data) => {
                        self.image_panel.images.insert(ImageType::Processed, image_panel_data);
                        self.panel_information.image_process_result = String::from("OK");
                        self.panel_information.processed_type = self.user_interact_items.process_type;
                        self.panel_information.image_width = PanelInfoImageValueState::Set(self.image_panel.images[&ImageType::Processed].get_image_width());
                        self.panel_information.image_height = PanelInfoImageValueState::Set(self.image_panel.images[&ImageType::Processed].get_image_height());
                        self.panel_information.max_image_value = PanelInfoImageValueState::Set(self.image_panel.images[&ImageType::Processed].get_max_image_value());
                        self.panel_information.min_image_value = PanelInfoImageValueState::Set(self.image_panel.images[&ImageType::Processed].get_min_image_value());
                    },
                    Err(e) => {
                        self.panel_information.processed_type = ProcessType::None;
                        self.panel_information.image_process_result = e.message;
                        self.panel_information.image_width = PanelInfoImageValueState::Unset;
                        self.panel_information.image_height = PanelInfoImageValueState::Unset;
                        self.panel_information.max_image_value = PanelInfoImageValueState::Unset;
                        self.panel_information.min_image_value = PanelInfoImageValueState::Unset;
                    }
                }
                Command::none()
            }
        }
    }


}

