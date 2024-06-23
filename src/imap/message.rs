pub enum Data {
    Atom(String),
    Number(i32),
    String(String),
    Nil,
}

impl Data {
    fn str_to_list(args_str: &str) -> Vec<Data> {
        let mut datas = Vec::new();
        let mut buf = String::new();
        let mut in_quotes = false;
        for c in args_str.chars() {
            match c {
                '"' => {
                    if in_quotes {
                        datas.push(Data::String(buf));
                        buf = String::new();
                    }
                    in_quotes = !in_quotes;
                }
                ' ' if !in_quotes => {
                    if !buf.is_empty() {
                        let data = if let Ok(n) = buf.parse() {
                            Data::Number(n)
                        } else if buf == "nil" {
                            Data::Nil
                        } else {
                            Data::Atom(buf)
                        };
                        datas.push(data);
                        buf = String::new();
                    }
                }
                _ => {
                    buf.push(c);
                }
            }
        }
        datas
    }

    pub fn into_string(self) -> String {
        match self {
            Data::Atom(s) => s,
            Data::String(s) => s,
            Data::Number(n) => n.to_string(),
            Data::Nil => "nil".into(),
        }
    }
}

impl ToString for Data {
    fn to_string(&self) -> String {
        match self {
            Data::Atom(s) => s.clone(),
            Data::String(s) => s.clone(),
            Data::Number(n) => n.to_string(),
            Data::Nil => "nil".into(),
        }
    }
}

pub enum Message {
    Capability {
        id: String,
    },
    Login {
        id: String,
        email: String,
        password: String,
    },
    NoOp {
        id: String,
    },
    End,
    Unknown {
        id: String,
        command: String,
        args: Vec<Data>,
    },
    Bad(String),
}

impl Message {
    pub fn parse(message: String) -> Self {
        if message.len() <= 2 {
            return Message::End;
        }
        let message = message[..(message.len() - 2)].to_string();
        if message.is_empty() {
            return Message::End;
        }

        let Some((id, body)) = message.split_once(' ') else {
            return Message::Bad(message);
        };

        let (command, args) = {
            let mut it = body.splitn(2, ' ');
            let command = it.next().unwrap();
            let args_str = it.next().unwrap_or_default();
            let args = Data::str_to_list(args_str);
            (command, args)
        };

        match command {
            "CAPABILITY" => Message::Capability { id: id.into() },
            "LOGIN" => {
                let mut it = args.into_iter();
                let email = it.next();
                let password = it.next();
                let Some((email, password)) = email.zip(password) else {
                    return Message::Bad(message);
                };
                Message::Login {
                    id: id.into(),
                    email: email.into_string(),
                    password: password.into_string(),
                }
            }
            "NOOP" => Message::NoOp { id: id.into() },
            _ => Message::Unknown {
                id: id.into(),
                command: command.into(),
                args,
            },
        }
    }
}
