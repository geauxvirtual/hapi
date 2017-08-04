use std::fs;
use std::path::Path;

use rocket::{Request, Data, Outcome};
use rocket::data::{self, FromData};
use rocket::http::Status;

use multipart::server::{Multipart, SavedFile, MultipartData};
use multipart::server::SaveResult::{Full, Partial, Error};

pub struct ActivityRequest {
    pub file: SavedFile,
    pub data_type: String,
    pub name: Option<String>,
    pub activity_type: Option<String>,
}

impl FromData for ActivityRequest {
    type Error = ();

    fn from_data(request: &Request,
                 data: Data) -> data::Outcome<Self, Self::Error> {
        // Reject request if Content-Length is not set
        let cl = match request.headers().get_one("Content-Length") {
            Some(val) => val,
            None => return Outcome::Failure((Status::LengthRequired, ())),
        };
        // Reject request if Content-Length is > 10MB
        if cl.parse::<i32>().unwrap() > 10 * 1024 * 1024 {
            return Outcome::Failure((Status::PayloadTooLarge, ()));
        }
        
        let ct = match request.headers().get_one("Content-Type") {
            Some(val) => val,
            None => return Outcome::Failure((Status::BadRequest, ())),
        };
        let idx = match ct.find("boundary=") {
            Some(val) => val,
            None => return Outcome::Failure((Status::BadRequest, ())),
        };
        let boundary = &ct[(idx + "boundary=".len())..];

        let mut mp = Multipart::with_body(data.open(), boundary);

        // Let's process the received Multipart entries. We currently
        // only support receiving one file per request. Each "file" field
        // should be accompanied by a "data_type" field. "name" and
        // "activity_type" fields are optional. We will loop through
        // the multipart request keeping track of the number of files
        // received. If more than one file received, the request will be
        // returned with an error.

        let mut count = 0;

        let mut sf = None;
        let mut data_type = None;
        let mut name = None;
        let mut activity_type = None;

        loop {
            let field = match mp.read_entry() {
                Ok(Some(field)) => field,
                Ok(None) => break,
                Err(_) => return Outcome::Failure((Status::BadRequest, ())),
            };

            match field.data {
                MultipartData::File(mut file) => {
                    if count >= 1 {
                        if sf.is_some() {
                            remove_file(sf.unwrap());
                        }
                        return Outcome::Failure((Status::BadRequest, ()));
                    }
                    count += 1;
                    match file.save()
                              .size_limit(1 * 1024 * 1024)
                              .temp()
                    {
                        Full(saved_file) => {
                            sf = Some(saved_file);
                        },
                        Partial(partial, _) => {
                            remove_file(partial);
                            return Outcome::Failure((Status::PayloadTooLarge, ()));
                        }
                        Error(_) => return Outcome::Failure((Status::InternalServerError, ())),
                    }
                },
                MultipartData::Text(text) => {
                    match field.name.as_str() {
                        "data_type" => data_type = Some(text.text.into()),
                        "name" => name = Some(text.text.into()),
                        "activity_type" => activity_type = Some(text.text.into()),
                        _ => return Outcome::Failure((Status::BadRequest, ())),
                    }
                }
            }
        }
        if sf.is_none() {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        if data_type.is_none() {
            remove_file(sf.unwrap());
            return Outcome::Failure((Status::BadRequest, ()));
        }

        Outcome::Success(ActivityRequest {
            file: sf.unwrap(),
            data_type: data_type.unwrap(),
            name: name,
            activity_type: activity_type,
        })
    }
}

pub fn create_dir(path: &Path) {
    let _ = fs::create_dir_all(path);
}

pub fn remove_file(file: SavedFile) {
    println!("Removing file: {}", file.path.to_str().unwrap());
    let _ = fs::remove_file(&file.path);
}
