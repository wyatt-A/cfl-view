use array_lib::ArrayDim;
use array_lib::cfl::ndarray::CowRepr::View;
use array_lib::cfl::num_complex::Complex32;


const DEFAULT_DIMS:usize = 128;

struct AppState {
    /// full cfl array
    cfl_buffer:CflBuffer,
    scale_handler: ScaleHandler,
}

impl Default for AppState {
    fn default() -> AppState {
        AppState {
            cfl_buffer: Default::default(),
            scale_handler: Default::default(),
        }
    }
}

enum ViewMode {
    Re,
    Im,
    Mag,
    Phase
}

impl Default for ViewMode {
    fn default() -> ViewMode {
        ViewMode::Mag
    }
}

/// controls how the cfl is rendered to the display
struct ScaleHandler {
    view_mode:ViewMode,
    scaling: f32,
}

impl Default for ScaleHandler {
    fn default() -> ScaleHandler {
        ScaleHandler {
            view_mode: ViewMode::default(),
            scaling: u8::MAX as f32,
        }
    }
}

impl ScaleHandler {

    pub fn calc_rgba(&self,data:&[Complex32]) -> Vec<u8> {

        // 4 bytes per sample
        let mut bytes = vec![0u8; data.len() * 4];

        bytes.chunks_mut(4).zip(data).for_each(|(rgba,sample)|{
                match self.view_mode {
                    ViewMode::Mag => {
                        let x = (sample.norm() * self.scaling).clamp(0.,255.) as u8;
                        rgba[0..3].fill(x);
                        rgba[3] = 255;
                    }
                    _=> todo!()
                }
        });

        bytes

    }


}



struct CflBuffer {
    data: Vec<Complex32>,
    dims: ArrayDim,
}

impl Default for CflBuffer {

    /// default buffer holds a blank 128x128 image
    fn default() -> CflBuffer {
        let dims = ArrayDim::from_shape(&[DEFAULT_DIMS,DEFAULT_DIMS]);
        CflBuffer {
            data: vec![Complex32::ZERO;dims.numel()],
            dims,
        }
    }
}


/// determines which cfl slices are displayed
struct SliceHandler {
    /// the cfl dimensions corresponding to the view (x,y,z) dims
    view_slices: [usize;3],
    /// the slices to render from the cfl
    slice_indices: [usize;3],
    slice_view_1: Vec<Complex32>,
    slice_view_2: Vec<Complex32>,
    slice_view_3: Vec<Complex32>,
    dims_1: ArrayDim,
    dims_2: ArrayDim,
    dims_3: ArrayDim,
}

impl From<ArrayDim> for SliceHandler {
    fn from(dims: ArrayDim) -> SliceHandler {
        let mut sh = SliceHandler::default();
        let shape = dims.shape();
        // center the view for 'x' 'y' 'z'

        let dx = shape[sh.view_slices[0]];
        let dy = shape[sh.view_slices[1]];
        let dz = shape[sh.view_slices[2]];

        let slice_idx_x = dx / 2;
        let slice_idx_y = dy / 2;
        let slice_idx_z = dz / 2;

        sh.slice_indices[0] = slice_idx_x;
        sh.slice_indices[1] = slice_idx_y;
        sh.slice_indices[2] = slice_idx_z;

        sh.dims_1 = ArrayDim::from_shape(&[dx,dy]);
        sh.dims_2 = ArrayDim::from_shape(&[dy,dz]);
        sh.dims_3 = ArrayDim::from_shape(&[dz,dx]);

        sh.slice_view_1 = vec![Complex32::ZERO;sh.dims_1.numel()];
        sh.slice_view_2 = vec![Complex32::ZERO;sh.dims_2.numel()];
        sh.slice_view_3 = vec![Complex32::ZERO;sh.dims_3.numel()];

        sh
    }
}


impl SliceHandler {

    /// updates the internal slice buffers based on the current view and slice indices
    fn update_slice_1(&mut self, cfl_buffer: &CflBuffer) -> Result<(), ()> {
        let mut idx = [0usize;16];
        let mut c = 0;
        // loop over x-y plane
        for y in 0..cfl_buffer.dims.shape()[self.view_slices[1]] {
            idx[self.view_slices[1]] = y;
            for x in 0..cfl_buffer.dims.shape()[self.view_slices[0]] {
                idx[self.view_slices[0]] = x;
                let addr = cfl_buffer.dims.calc_addr(&idx);
                self.slice_view_1[c] = cfl_buffer.data[addr];
                c += 1;
            }
        }
        Ok(())
    }

    fn update_slice_2(&mut self, cfl_buffer: &CflBuffer) -> Result<(), ()> {
        let mut idx = [0usize;16];
        let mut c = 0;
        // loop over y-z plane
        for z in 0..cfl_buffer.dims.shape()[self.view_slices[2]] {
            idx[self.view_slices[2]] = z;
            for y in 0..cfl_buffer.dims.shape()[self.view_slices[1]] {
                idx[self.view_slices[1]] = y;
                let addr = cfl_buffer.dims.calc_addr(&idx);
                self.slice_view_2[c] = cfl_buffer.data[addr];
                c += 1;
            }
        }
        Ok(())
    }

    fn update_slice_3(&mut self, cfl_buffer: &CflBuffer) -> Result<(), ()> {
        let mut idx = [0usize;16];
        let mut c = 0;
        // loop over z-x plane
        for x in 0..cfl_buffer.dims.shape()[self.view_slices[0]] {
            idx[self.view_slices[0]] = x;
            for z in 0..cfl_buffer.dims.shape()[self.view_slices[2]] {
                idx[self.view_slices[2]] = z;
                let addr = cfl_buffer.dims.calc_addr(&idx);
                self.slice_view_3[c] = cfl_buffer.data[addr];
                c += 1;
            }
        }
        Ok(())
    }
}

impl Default for SliceHandler {
    fn default() -> SliceHandler {
        SliceHandler {
            view_slices: [0,1,2],
            slice_indices: [0,0,0],
            slice_view_1: vec![Complex32::ZERO;DEFAULT_DIMS*DEFAULT_DIMS],
            slice_view_2: vec![Complex32::ZERO;DEFAULT_DIMS*DEFAULT_DIMS],
            slice_view_3: vec![Complex32::ZERO;DEFAULT_DIMS*DEFAULT_DIMS],
            dims_1: ArrayDim::from_shape(&[DEFAULT_DIMS, DEFAULT_DIMS]),
            dims_2: ArrayDim::from_shape(&[DEFAULT_DIMS, DEFAULT_DIMS]),
            dims_3: ArrayDim::from_shape(&[DEFAULT_DIMS, DEFAULT_DIMS]),
        }
    }
}







fn main() {

    let cfl_buffer = CflBuffer {
        data: vec![Complex32::ZERO;128*512*64],
        dims: ArrayDim::from_shape(&[128, 512, 64]),
    };

    let sch = ScaleHandler::default();

    let mut sh = SliceHandler::from(cfl_buffer.dims);

    sh.update_slice_1(&cfl_buffer).unwrap();
    sh.update_slice_2(&cfl_buffer).unwrap();
    sh.update_slice_3(&cfl_buffer).unwrap();

    sch.calc_rgba(&sh.slice_view_1);
    sch.calc_rgba(&sh.slice_view_2);
    sch.calc_rgba(&sh.slice_view_3);

}