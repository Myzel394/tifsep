struct UserAgent(String);

// impl<'a, 'r> FromRequest<'a, 'r> for Token {
//     type Error = Infallible;
//
//     fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
//         let token = request.headers().get_one("token");
//         match token {
//           Some(token) => {
//             // check validity
//             Outcome::Success(Token(token.to_string()))
//           },
//           // token does not exist
//           None => Outcome::Failure(Status::Unauthorized)
//         }
//     }
// }
