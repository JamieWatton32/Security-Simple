
// use passwords::PasswordGenerator;
// use std::io::{stdin, stdout, Write};



// pub struct Commands<'a> {
//     pub password_manager: Database<'a>,
// }
// impl<'a> Commands<'a> {
//     pub fn new(password_manager: Database<'a>) -> Self {
//         Self { password_manager }
//     }

//     pub fn get_input(prompt: &str) -> String {
//         print!("{}", prompt);
//         stdout().flush().expect("Failed to flush stdout");

//         let mut user_input = String::new();
//         stdin()
//             .read_line(&mut user_input)
//             .expect("Failed to read user input");
//         user_input.trim().to_ascii_lowercase()
//     }

//     pub fn add_password(&self) {
//         'add: loop {
//             let site_name = Self::get_input("Enter the site name (or 'quit' to end): ");
//             if site_name == "quit" {
//                 break 'add;
//             }

//             let mut password = Self::get_input("Enter the password or generate one! ");
//             if password == "generate" {
//                 password = Self::generate_password();
//             }
//             match self.password_manager.add(&site_name, &password) {
//                 Ok(_) => println!("Password added successfully"),
//                 Err(e) => eprintln!("Failed to add password: {:?}", e),
//             }

//             if Self::get_input("Would you like to add another? (yes/no): ") != "yes" {
//                 break 'add;
//             }
//         }
//     }

//     pub fn get_password(&self) {
//         let site_name = Self::get_input("Enter the site name to retrieve password: ");
//         match self.password_manager.get(&site_name) {
//             Ok(pwd) => println!("Retrieved password: {:?}", pwd),
//             Err(e) => eprintln!("Failed to retrieve password: {:?}", e),
//         }
//     }

//     pub fn drop_password(&self) {
//         let site_name = Self::get_input("Enter the site name to drop password: ");
//         match self.password_manager.drop(&site_name) {
//             Ok(_) => println!("Password dropped successfully"),
//             Err(e) => eprintln!("Failed to drop password: {:?}", e),
//         }
//     }

//     fn generate_password() -> String {
//         let pg = PasswordGenerator::new()
//             .length(24)
//             .numbers(true)
//             .lowercase_letters(true)
//             .uppercase_letters(true)
//             .symbols(true)
//             .spaces(true)
//             .exclude_similar_characters(true)
//             .strict(true);
        
//         pg.generate_one().unwrap()
//     }
// }

 