use std::io;

use crate::{cons::Cons, prod::Prod};

struct MoonWalkRepr {
    coords: (i16, i16),
    dest_coords: (i16, i16),
    dim: i16,
    map_buf: Vec<u8>,
}

impl MoonWalkRepr {
    const RES: usize = 48;

    fn change_buf(&mut self, x: usize, y: usize, c: u8) {
        self.map_buf[(y * (MoonWalkRepr::RES + 2 + self.dim.to_string().len()) + x) as usize] = c;
    }

    fn new(mut x: i16, mut y: i16, mut x_dest: i16, mut y_dest: i16, dim: i16) -> MoonWalkRepr {
        let char_size =
            ((MoonWalkRepr::RES + 2 + dim.to_string().len()) * (MoonWalkRepr::RES + 2)) as usize;
        let dim_size = dim.to_string().len();

        let mut b = string_builder::Builder::new(char_size);

        for i in 0..MoonWalkRepr::RES {
            b.append(" ".repeat(MoonWalkRepr::RES));
            b.append("|");
            if i == 0 {
                b.append("1");
                b.append(" ".repeat(dim_size - 1));
            } else if i == MoonWalkRepr::RES - 1 {
                b.append(dim.to_string());
            } else {
                b.append(" ".repeat(dim_size));
            }
            b.append('\n');
        }

        b.append("-".repeat(MoonWalkRepr::RES as usize));
        b.append(" ".repeat(1 + dim_size));
        b.append('\n');

        b.append("1");
        b.append(" ".repeat(MoonWalkRepr::RES as usize - 1 - dim_size as usize));
        b.append(dim.to_string());
        b.append(" ".repeat(1 + dim_size));
        b.append('\n');

        let map_buf: Vec<u8> = b.string().unwrap().into_bytes();

        x = (x as f32 * (MoonWalkRepr::RES as f32 / dim as f32)) as i16;
        y = (y as f32 * (MoonWalkRepr::RES as f32 / dim as f32)) as i16;
        x_dest = (x_dest as f32 * (MoonWalkRepr::RES as f32 / dim as f32)) as i16;
        y_dest = (y_dest as f32 * (MoonWalkRepr::RES as f32 / dim as f32)) as i16;

        let mut res = MoonWalkRepr {
            coords: (x, y),
            dest_coords: (x_dest, y_dest),
            dim: dim,
            map_buf: map_buf,
        };

        if res.coords == res.dest_coords {
            res.change_buf(res.coords.0 as usize, res.coords.1 as usize, 'Q' as u8);
        } else {
            res.change_buf(res.coords.0 as usize, res.coords.1 as usize, 'O' as u8);
            res.change_buf(
                res.dest_coords.0 as usize,
                res.dest_coords.1 as usize,
                'X' as u8,
            );
        }

        return res;
    }

    fn set_pos(&mut self, x: i16, y: i16) {
        if (x, y) != self.coords {
            if self.coords == self.dest_coords {
                //print!("pos {} {} {} {} X", x, y, self.coords.0, self.coords.1);
                self.change_buf(self.coords.0 as usize, self.coords.1 as usize, 'X' as u8);
            } else {
                //print!("pos {} {} {} {}  ", x, y, self.coords.0, self.coords.1);
                self.change_buf(self.coords.0 as usize, self.coords.1 as usize, ' ' as u8);
            }
            if (x, y) == self.dest_coords {
                //print!("pos {} {} {} {} Q", x, y, self.coords.0, self.coords.1);
                self.change_buf(x as usize, y as usize, 'Q' as u8);
            } else {
                //print!("pos {} {} {} {} O", x, y, self.coords.0, self.coords.1);
                self.change_buf(x as usize, y as usize, 'O' as u8);
            }

            self.coords = (x, y);
        }
    }

    fn set_dest(&mut self, x_dest: i16, y_dest: i16) {
        if (x_dest, y_dest) != self.dest_coords {
            if self.coords == self.dest_coords {
                //print!("dest {} {} O", x_dest, y_dest);
                self.change_buf(self.coords.0 as usize, self.coords.1 as usize, 'O' as u8);
            } else {
                //print!("dest {} {}  ", x_dest, y_dest);
                self.change_buf(
                    self.dest_coords.0 as usize,
                    self.dest_coords.1 as usize,
                    ' ' as u8,
                );
            }
            if (x_dest, y_dest) == self.coords {
                //print!("dest {} {} Q", x_dest, y_dest);
                self.change_buf(x_dest as usize, y_dest as usize, 'Q' as u8);
            } else {
                //print!("dest {} {} X", x_dest, y_dest);
                self.change_buf(x_dest as usize, y_dest as usize, 'X' as u8);
            }

            self.dest_coords = (x_dest, y_dest);
        }
    }

    fn draw(&self) {
        unsafe {
            print!("{}", String::from_utf8_unchecked(self.map_buf.to_vec()));
        }
    }

    fn update_stuff(&mut self, mut x: i16, mut y: i16, mut x_dest: i16, mut y_dest: i16) -> bool {
        x = (x as f32 * (MoonWalkRepr::RES as f32 / self.dim as f32)) as i16;
        y = (y as f32 * (MoonWalkRepr::RES as f32 / self.dim as f32)) as i16;
        x_dest = (x_dest as f32 * (MoonWalkRepr::RES as f32 / self.dim as f32)) as i16;
        y_dest = (y_dest as f32 * (MoonWalkRepr::RES as f32 / self.dim as f32)) as i16;

        let mut res = false;
        if (x, y) != self.coords || (x_dest, y_dest) != self.dest_coords {
            res = true;
        }
        self.set_pos(x, y);
        self.set_dest(x_dest, y_dest);

        return res;
    }

    fn erase(&self) {
        print!("\x1B[{}A", MoonWalkRepr::RES + 2);
    }
}

pub struct ClientUI {
    cons: Cons,
    repr: MoonWalkRepr,
}

impl ClientUI {
    async fn new() -> Self {
        let mut cons = Cons::new("server-client-out").await;
        let mut message = cons.read().await;
        let dim: i16;
        unsafe {
            dim = String::from_utf8_unchecked(message[2..].to_vec())
                .parse()
                .unwrap();
        }

        cons.move_offset_curr().await;
        message = cons.read().await;

        let args = ClientUI::parse_position(&message);
        let res = ClientUI {
            cons: cons,
            repr: MoonWalkRepr::new(args.0, args.1, args.2, args.3, dim),
        };

        res.repr.draw();
        return res;
    }

    fn parse_position(message: &Vec<u8>) -> (i16, i16, i16, i16) {
        let mut vals = message[2..].split(|x| x.to_owned() == ',' as u8);
        let (x, y, x_dest, y_dest): (i16, i16, i16, i16);
        unsafe {
            x = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();
            y = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();
            x_dest = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();
            y_dest = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();
        }
        (x, y, x_dest, y_dest)
    }

    pub async fn run_client_ui() {
        let mut client = ClientUI::new().await;
        loop {
            let message = client.cons.read().await;
            match message[0] as char {
                's' => {
                    let func_args = ClientUI::parse_position(&message);
                    if message.len() != 0 {
                        if client.repr.update_stuff(
                            func_args.0,
                            func_args.1,
                            func_args.2,
                            func_args.3,
                        ) {
                            client.repr.erase();
                            client.repr.draw();
                        }
                    }
                }
                'p' => {}
                _ => {}
            }
        }
    }
}

pub struct ClientControl {
    voter: Prod,
}

impl ClientControl {
    async fn new() -> Self {
        ClientControl {
            voter: Prod::new("client-votes").await,
        }
    }

    pub async fn run_client_control() {
        let client_control = ClientControl::new().await;
        println!("Command away:");
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.pop();
            let mut nums = input.split(|x| x == ' ');
            if let (Some(first), Some(second)) = (nums.next(), nums.next()) {
                if let (Ok(_), Ok(_)) = (first.parse::<i16>(), second.parse::<i16>()) {
                    client_control.voter.send(&input).await;
                    println!("sent a message to the server: {}", &input);
                } else {
                    println!("Ok, we need two numbers");
                }
            } else {
                println!("Ok, we need two numbers");
            }
        }
    }
}
