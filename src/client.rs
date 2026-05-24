use crate::cons::Cons;

struct MoonWalkRepr {
    coords: (i16, i16),
    dest_coords: (i16, i16),
    dim: i16,
    char_size: usize,
    eraser: String,
    dim_size: usize,
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
        let eraser = "\x08".repeat(char_size);
        let dim_size = dim.to_string().len();

        let mut b = string_builder::Builder::new(char_size);

        for i in 0..dim {
            b.append(" ".repeat(dim as usize));
            b.append("|");
            if i == 0 {
                b.append("0");
                b.append(" ".repeat(dim_size - 1));
            } else if i == dim - 1 {
                b.append(i.to_string());
            } else {
                b.append(" ".repeat(dim_size));
            }
            b.append('\n');
        }

        b.append("-".repeat(dim as usize));
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
            char_size: char_size,
            eraser: eraser,
            dim_size: dim_size,
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
                self.change_buf(self.coords.0 as usize, self.coords.1 as usize, 'X' as u8);
                self.change_buf(x as usize, y as usize, 'O' as u8);
            }
            self.coords = (x, y);
        }
    }

    fn set_dest(&mut self, x_dest: i16, y_dest: i16) {
        if (x_dest, y_dest) != self.dest_coords {
            if self.coords == self.dest_coords {
                self.change_buf(
                    self.dest_coords.0 as usize,
                    self.dest_coords.1 as usize,
                    'O' as u8,
                );
                self.change_buf(x_dest as usize, y_dest as usize, 'X' as u8);
            }
            self.coords = (x_dest, y_dest);
        }
    }

    fn draw(&self) {
        unsafe {
            print!("{}", String::from_utf8_unchecked(self.map_buf.to_vec()));
        }
    }

    fn erase(&self) {
        print!("{}", self.eraser);
    }
}

pub struct Client {
    cons: Cons,
    repr: MoonWalkRepr,
}

impl Client {
    async fn new() -> Self {
        let mut cons = Cons::new("moon-walker-out").await;
        let mut msg = cons.read().await;
        let dim: i16;
        unsafe {
            dim = String::from_utf8_unchecked(msg[2..].to_vec())
                .parse()
                .unwrap();
        }

        cons.move_offset_curr().await;
        msg = cons.read().await;
        while msg.len() == 0 || msg[0] != 's' as u8 {
            msg = cons.read().await;
        }

        let mut vals = msg[2..].split(|x| x.to_owned() == ',' as u8);
        unsafe {
            let x: i16 = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();
            let y: i16 = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();
            let x_dest: i16 = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();
            let y_dest: i16 = String::from_utf8_unchecked(vals.next().unwrap().to_vec())
                .parse()
                .unwrap();

            Client {
                cons: cons,
                repr: MoonWalkRepr::new(x, y, x_dest, y_dest, dim),
            }
        }
    }

    // Don't look at how inefficient this is, I know it is

    pub async fn run_client() {
        let mut client = Client::new().await;
        client.repr.draw();
        loop {
            let msg = client.cons.read().await;
            if msg.len() != 0 {
                unsafe {
                    println!("{}", String::from_utf8_unchecked(msg.to_vec()));
                }
            }
        }
    }
}
