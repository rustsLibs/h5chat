use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use iron::headers::Cookie;
use cookie::Cookie as CookieObj;
use std::ops::Deref;
use redis::Commands;

use persistent::Read as PersistRead;
use iron::modifiers::Redirect;
use iron::Url;
use iron::status;
use uuid::Uuid;

use AppRedis;

pub struct CheckLogin;
impl typemap::Key for CheckLogin { type Value = bool; }
pub struct UserCookie;
impl typemap::Key for UserCookie { type Value = String; }
pub struct ManagerId;
impl typemap::Key for ManagerId { type Value = Uuid; }




impl BeforeMiddleware for CheckLogin {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let pool = req.get::<PersistRead<AppRedis>>().unwrap();
        let redis_client_wr = pool.get().unwrap();
        let redis_client = redis_client_wr.deref();
        let conn = redis_client.get_connection().unwrap();
        
        // get the cookie part in the request headers
        let cookiep =  req.headers.get::<Cookie>();
        println!("{:?}", cookiep);
        match cookiep {
            Some(ref value) => {
                //println!("{:?}", value);
                let Cookie(ref ckvec) = **value;
                //println!("{:?}", ckvec);
                let cookie_vec = ckvec.iter()
                                    .filter(|item: &&CookieObj| item.name == "h5chat_ci".to_owned())
                                    .take(1)
                                    .collect::<Vec<&CookieObj>>();
                //println!("{:?}", cookie_vec);
                let cookie_obj = cookie_vec[0];
                //println!("{:?}", cookie_obj);
                let cookie_value = cookie_obj.value.clone();
                //println!("{:?}", cookie_value);
                
                // pass this cookie_value to redis 
                let cookie_key = "UserCookie:".to_string() + &cookie_value;
                let manager_id = conn.get(cookie_key).unwrap_or("".to_owned());
                if manager_id != "".to_owned() {
                    println!("has logined");
                    req.extensions.insert::<CheckLogin>(true);
                    // retreive the uuid string from redis, convert to Uuid obj
                    req.extensions.insert::<ManagerId>(Uuid::parse_str(&manager_id).unwrap());
                }
                else {
                    println!("not login");
                    req.extensions.insert::<CheckLogin>(false);
                }
                
                // save user's cookie
                req.extensions.insert::<UserCookie>(cookie_value);
                
            },
            None => {
                println!("no cookie");
                req.extensions.insert::<CheckLogin>(false);
            }
        }
        
        Ok(())
    }
}

impl AfterMiddleware for CheckLogin {
    fn after(&self, req: &mut Request, mut res: Response) -> IronResult<Response> {
        let logined = req.extensions.get::<CheckLogin>().unwrap();

        // if logined, when click login button, enter board.html directly
        println!("{:?}", req.url.path);
        let path = &req.url.path;
        if *logined {
			// when logined, directly enter board view
			if path[0] == "page".to_owned() 
                && path[1] == "login.html".to_owned() {
				println!("I have logined");
				let url = Url::parse("http://127.0.0.1:8080/page/board/index.html").unwrap();
				res.set_mut(status::Found).set_mut(Redirect(url));
			}
        }
        else {
            // not logined but in board pages, return to index frontpage
            println!("{:?}", "in after ware, not login.");
            if path[0] == "page".to_owned() && path[1] == "board".to_owned(){
                let url = Url::parse("http://127.0.0.1:8080/page/index.html").unwrap();
                res.set_mut(status::Found).set_mut(Redirect(url));
            }
            
        }
        
        
        Ok(res)
    }
}



