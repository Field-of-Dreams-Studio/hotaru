use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::Cors; 

pub static APP: SApp = Lazy::new(|| App::new().binding("127.0.0.1:3031").build());

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}

mod resource;

endpoint! {
    APP.url("/"),
    middleware = [Print],
    config = [HttpSafety::default()], 

    /// # Request 
    /// 
    /// `GET /` 
    /// 
    /// # Response 
    /// 
    /// `TEXT Hello From Hotaru 0.7` 
    /// 
    /// # Comments 
    /// 
    /// This is a doc comment 
    /// 
    /// Some IDEs may emit an error when you write this but it can be parsed successfully 
    /// 
    /// If you add an pub then you may use this as an function 
    pub hello <HTTP> {
        text_response("Hello From Hotaru 0.7!")
    }
} 

endpoint! { 
    APP.url("/root_cloned"), 

    /// You may add different middlewares and configs 
    pub hello_cloned <HTTP> { 
        hello(req).await 
    }
}

endpoint! {
    APP.url("/<int:id>/<app_name>"),
    middleware = [Print],
    config = [HttpSafety::default()], 

    pub pattern <HTTP> {
        let id = req.pattern("id").unwrap();
        let app_name = req.pattern("app_name").unwrap();
        json_response(object!({
            id: id,
            app_name: app_name
        }))
    }
}

endpoint! {
    APP.url("/anno"), 
    
    _ <HTTP> {
        text_response("Hello From Hotaru 0.7!")
    }
}

middleware! {
    pub Print <HTTP> {
        println!("Request received");
        next(req).await
    }
}
