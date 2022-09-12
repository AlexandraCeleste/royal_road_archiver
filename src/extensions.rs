pub trait VecStringExt<String> { // Honestly this whole thing is completely pointless, I was experimenting.
    fn remove_duplicates(&self) -> Vec<String>;
}

impl VecStringExt<String> for Vec<String> {
    fn remove_duplicates(&self) -> Vec<String> {
        let mut seen: Vec<String> = Vec::with_capacity(self.len());
        let mut output: Vec<String> = Vec::with_capacity(self.len());

        for i in 0..self.len() {
            if seen.contains(&self[i]) { continue; }
            else { seen.push(self[i].clone()); output.push(self[i].clone()); }
        }

        return output;
    }
}