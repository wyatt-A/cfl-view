// use std::fmt::{Debug, Formatter};
// use std::path::PathBuf;
// use std::time::Instant;
// use array_lib::ArrayDim;
// use array_lib::cfl::num_complex::Complex32;
// use array_lib::io_cfl::read_cfl;
// use rfd::FileDialog;
// use crate::ViewError::BadIndex;
// use rayon::prelude::*;
//
//
// use iced::{
//     Element, Length,
//     Application, Settings, Theme,
//     widget::{image, column}
// };
// use iced::widget::{button, row, Column};
//
// pub fn main() -> iced::Result {
//     iced::run(GrayApp::update,GrayApp::view)
// }
//
// struct GrayApp {
//     view_buffer: Option<ViewBuffer>,
//     image_path: Option<PathBuf>,
//     handle_x: image::Handle,
//     handle_y: image::Handle,
//     handle_z: image::Handle,
// }
//
// impl Default for GrayApp {
//     fn default() -> Self {
//         GrayApp::new(ArrayDim::from_shape(&[128,128]))
//     }
// }
//
//
// impl GrayApp {
//     fn new(dims:ArrayDim) -> Self {
//         let shape = dims.shape();
//         let disp_buffer = vec![0u8;dims.numel() * 4];
//         let handle_x = image::Handle::from_rgba(shape[0] as u32, shape[1] as u32, disp_buffer.clone());
//         let handle_y = image::Handle::from_rgba(shape[0] as u32, shape[1] as u32, disp_buffer.clone());
//         let handle_z = image::Handle::from_rgba(shape[0] as u32, shape[1] as u32, disp_buffer);
//         Self { handle_x,handle_y,handle_z,image_path:None,view_buffer:None }
//     }
// }
//
// #[derive(Debug, Clone)]
// enum Message {
//     LoadImage
// }
//
// impl GrayApp {
//     pub fn view(&self) -> Column<Message> {
//         // We use a column: a simple vertical layout
//         let img_x = image(&self.handle_x)
//             .width(Length::Shrink)
//             .height(Length::Shrink);
//
//         let img_y = image(&self.handle_y)
//             .width(Length::Shrink)
//             .height(Length::Shrink);
//
//         let img_z = image(&self.handle_z)
//             .width(Length::Shrink)
//             .height(Length::Shrink);
//
//         let button = button("load image").on_press(Message::LoadImage);
//
//
//
//
//         column![
//             button,
//             row![
//                 img_x,img_y,img_z
//             ]
//
//         ].padding(20)
//     }
//
//     pub fn update(&mut self, message: Message) {
//         match message {
//             Message::LoadImage => {
//                 if let Some(path) = pick_file() {
//                     let (data,dims) = read_cfl(&path);
//                     self.image_path = Some(path);
//                     if let Ok(mut view_buffer) = ViewBuffer::new(CflBuffer {data,dims},[0,1,2]) {
//
//
//                         let (data,dims) = view_buffer.extract_slice_x();
//                         let shape = dims.shape();
//                         let img_data = data.iter().map(|val| {
//                             let x = 10 * val.norm() as u8;
//                             [x,x,x,255]
//                         }).flatten().collect::<Vec<u8>>();
//                         self.handle_x = image::Handle::from_rgba(shape[0] as u32, shape[1] as u32, img_data);
//
//                         let (data,dims) = view_buffer.extract_slice_y();
//                         let shape = dims.shape();
//                         let img_data = data.iter().map(|val| {
//                             let x = 10 * val.norm() as u8;
//                             [x,x,x,255]
//                         }).flatten().collect::<Vec<u8>>();
//                         self.handle_y = image::Handle::from_rgba(shape[0] as u32, shape[1] as u32, img_data);
//
//
//                         let (data,dims) = view_buffer.extract_slice_z();
//                         let shape = dims.shape();
//                         let img_data = data.iter().map(|val| {
//                             let x = 10 * val.norm() as u8;
//                             [x,x,x,255]
//                         }).flatten().collect::<Vec<u8>>();
//                         self.handle_z = image::Handle::from_rgba(shape[0] as u32, shape[1] as u32, img_data);
//
//                         self.view_buffer = Some(view_buffer);
//                     }
//                 }
//             }
//         }
//     }
// }
//
// // fn main() -> Result<(),ViewError> {
// //
// //     if let Some(cfl) = pick_file() {
// //
// //         let (data,dims) = read_cfl(&cfl);
// //         println!("cfl loaded");
// //         println!("{:?}",dims);
// //
// //         let buffer = CflBuffer {
// //             data,
// //             dims,
// //         };
// //
// //         let mut vb = ViewBuffer::new(&buffer,[0,1,2])?;
// //
// //         let now = Instant::now();
// //         let (slice_x,..) = vb.extract_slice_x();
// //         let (slice_y,..) = vb.extract_slice_y();
// //         let (slice_z,..) = vb.extract_slice_z();
// //         let dur = now.elapsed();
// //         //println!("{:?}",slice);
// //         println!("slice extraction took {} us",dur.as_micros());
// //
// //     }
// //
// //     Ok(())
// //
// // }
//
// #[derive(Debug)]
// enum ViewError {
//     BadIndex(usize),
// }
//
// fn pick_file() -> Option<PathBuf> {
//     FileDialog::new()
//         .add_filter("cfl_files", &["cfl", "hdr"])
//         .set_directory("/Users/wyatt")
//         .pick_file()
// }
//
//
// enum DisplayMode {
//     Mag,
//     Re,
//     Im,
//     Phase,
// }
//
// impl Default for DisplayMode {
//     fn default() -> DisplayMode {
//         DisplayMode::Mag
//     }
// }
//
// struct ViewPort {
//     display_mode: DisplayMode,
//
//
//
// }
//
// struct ViewBuffer {
//     slice_dims: [usize;3],
//     slice_idx: [usize;3],
//
//     cfl_buffer: CflBuffer,
//
//     view_dims_x: ArrayDim,
//     view_data_x: Vec<Complex32>,
//
//     view_dims_y: ArrayDim,
//     view_data_y: Vec<Complex32>,
//
//     view_dims_z: ArrayDim,
//     view_data_z: Vec<Complex32>,
// }
//
// impl Debug for ViewBuffer {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         writeln!(f,"slice_dims: {:?}",self.slice_dims)?;
//         writeln!(f,"slice_idx: {:?}",self.slice_idx)
//     }
// }
//
// impl ViewBuffer {
//
//
//     pub fn new(cfl_buffer:CflBuffer, slice_dims:[usize;3]) -> Result<ViewBuffer,ViewError> {
//         // check that the slice dims and indices are valid for cfl
//         let cfl_shape = cfl_buffer.dims.shape();
//         let mut slice_idx = [0,0,0];
//         for (&dim,i) in slice_dims.iter().zip(slice_idx.iter_mut()) {
//             let size = cfl_shape.get(dim).ok_or_else(|| BadIndex(dim))?;
//             *i = *size / 2;
//         }
//
//         let view_dims_x = ArrayDim::from_shape(
//             &[
//                 cfl_shape[slice_dims[0]],
//                 cfl_shape[slice_dims[1]],
//             ]
//         );
//
//         let view_dims_y = ArrayDim::from_shape(
//             &[
//                 cfl_shape[slice_dims[1]],
//                 cfl_shape[slice_dims[2]],
//             ]
//         );
//
//         let view_dims_z = ArrayDim::from_shape(
//             &[
//                 cfl_shape[slice_dims[2]],
//                 cfl_shape[slice_dims[0]],
//             ]
//         );
//
//         Ok(
//             ViewBuffer {
//                 slice_idx,
//                 slice_dims,
//                 cfl_buffer,
//                 view_dims_x,
//                 view_data_x: view_dims_x.alloc(Complex32::ZERO),
//                 view_dims_y,
//                 view_data_y: view_dims_x.alloc(Complex32::ZERO),
//                 view_dims_z,
//                 view_data_z: view_dims_x.alloc(Complex32::ZERO),
//             }
//         )
//     }
//
//     fn extract_slice_x(&mut self) -> (&[Complex32],ArrayDim) {
//
//
//         let cfl_strides = self.cfl_buffer.dims.strides();
//         let view_dims = self.view_dims_x.shape();
//
//         // fill the x-buffer
//         let mut cntr = 0;
//         for i in 0..view_dims[0] {
//             for j in 0..view_dims[1] {
//                 let addr = i * cfl_strides[self.slice_dims[0]] + j * cfl_strides[self.slice_dims[1]] + self.slice_idx[2] * cfl_strides[self.slice_dims[2]];
//                 self.view_data_x[cntr] = self.cfl_buffer.data[addr];
//                 cntr += 1;
//             }
//         }
//
//         (self.view_data_x.as_slice(),self.view_dims_x)
//
//     }
//
//     fn extract_slice_y(&mut self) -> (&[Complex32],ArrayDim) {
//
//
//         let cfl_strides = self.cfl_buffer.dims.strides();
//         let view_dims = self.view_dims_x.shape();
//
//         // fill the x-buffer
//         let mut cntr = 0;
//         for i in 0..view_dims[0] {
//             for j in 0..view_dims[1] {
//                 let addr = i * cfl_strides[self.slice_dims[1]] + j * cfl_strides[self.slice_dims[2]] + self.slice_idx[0] * cfl_strides[self.slice_dims[0]];
//                 self.view_data_x[cntr] = self.cfl_buffer.data[addr];
//                 cntr += 1;
//             }
//         }
//
//         (self.view_data_y.as_slice(),self.view_dims_y)
//
//     }
//
//     fn extract_slice_z(&mut self) -> (&[Complex32],ArrayDim) {
//
//
//         let cfl_strides = self.cfl_buffer.dims.strides();
//         let view_dims = self.view_dims_z.shape();
// 
//         // fill the x-buffer
//         let mut cntr = 0;
//         for i in 0..view_dims[0] {
//             for j in 0..view_dims[1] {
//                 let addr = i * cfl_strides[self.slice_dims[2]] + j * cfl_strides[self.slice_dims[0]] + self.slice_idx[1] * cfl_strides[self.slice_dims[1]];
//                 self.view_data_x[cntr] = self.cfl_buffer.data[addr];
//                 cntr += 1;
//             }
//         }
//
//         (self.view_data_x.as_slice(),self.view_dims_z)
//
//     }
//
//
// }
//
//
// struct CflBuffer {
//     data: Vec<Complex32>,
//     dims: ArrayDim,
// }