// use super::FutureResponse; 
// use hotaru_macro::middleware; 
// use hotaru::prelude::*; 

// #[middleware]
// pub async fn MyMiddleWare1(){ //Default param is req. Req should be mutable 
//     println!("Middleware: Received request for {}", req.path()); 
//     next(req)  
// }  

// #[middleware]
// pub async fn MyMiddleWare2(request_context: Rc){ // Ident name can be set 
//     println!("Middleware: Received request for {}", request_context.path()); 
//     next(request_context)  
// }  

// #[middleware]
// pub async fn MyMiddleWare3(request_context: Rc){ // Ident name can be set 
//     println!("Middleware: Received request for {}", request_context.path()); 
//     request_context.response = normal_response(StatusCode::BAD_GATEWAY, "111"); 
//     request_context.boxed_future() // Abropting the middleware chain 
// }  
