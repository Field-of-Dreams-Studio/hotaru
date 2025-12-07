// pub struct ParseConfig {
//     pub max_header_size: usize,
//     pub max_line_length: usize,
//     pub max_headers: usize,
//     pub max_body_size: usize, 
// } 

// impl ParseConfig {
//     pub fn new ( 
//         max_header_size: usize,
//         max_line_length: usize,
//         max_headers: usize,
//         max_body_size: usize,
//     ) -> Self {
//         Self {
//             max_header_size,
//             max_body_size,
//             max_line_length,
//             max_headers,
//         }
//     }

//     pub fn set_max_header_size(&mut self, size: usize) {
//         self.max_header_size = size;
//     }

//     pub fn set_max_body_size(&mut self, size: usize) {
//         self.max_body_size = size; 
//     }

//     pub fn set_max_line_length(&mut self, size: usize) {
//         self.max_line_length = size;
//     }

//     pub fn set_max_headers(&mut self, size: usize) {
//         self.max_headers = size;
//     }

//     pub fn get_max_header_size(&self) -> usize {
//         self.max_header_size
//     }

//     pub fn get_max_body_size(&self) -> usize {
//         self.max_body_size
//     }

//     pub fn get_max_line_length(&self) -> usize {
//         self.max_line_length
//     }

//     pub fn get_max_headers(&self) -> usize {
//         self.max_headers
//     }

//     pub fn default() -> Self {
//         Self {
//             max_header_size: 8192,
//             max_body_size: 1028 * 1028,
//             max_line_length: 8192,
//             max_headers: 100,
//         }
//     }
// } 
