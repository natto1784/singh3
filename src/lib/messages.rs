use serenity::model::channel::Message;

pub trait ExtractInfo {
    fn extract_text(&self, skip: usize, with_ref: bool) -> Option<String>;
}

impl ExtractInfo for Message {
    fn extract_text(&self, skip: usize, with_ref: bool) -> Option<String> {
        let mut ret: String = String::from("");

        let raw: Vec<&str> = self.content.splitn(skip + 1, " ").collect();
        if raw.len() == skip + 1 {
            ret += raw[skip];
            ret += "\n";
        } else if raw.len() < skip {
            return None;
        }

        ret += &self
            .attachments
            .iter()
            .map(|x| x.url.clone())
            .collect::<Vec<String>>()
            .join("\n");

            if let Some(msg) = &self.referenced_message {
                let ref_text = msg.extract_text(0, false);
                if with_ref && !ref_text.is_none() {
                    ret += "\n";
                    ret += &ref_text.unwrap();
                }
        }

        if ret.is_empty() {
            return None;
        } else {
            return Some(ret);
        }
    }
}
