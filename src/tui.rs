use std::io;

pub fn get_credentials() -> (String, String) {
    print!("username: ");
    let mut username = Default::default();
    io::stdin().read_line(&mut username).unwrap();

    print!("password: ");
    let mut password = Default::default();
    io::stdin().read_line(&mut password).unwrap();

    return (username, password);
}

pub fn print_menu(){
    println!("1: update ");
    println!("2: query");
}
