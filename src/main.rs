use std::cmp;
struct Trie { 
    data: Option<char>, 
    count: usize,
    children: Vec<Box<Trie>>,
}

impl Clone for Trie {
    fn clone(&self) -> Self {
        Trie {
            data: self.data,
            count: self.count,
            children: self.children.iter().map(|x| (x).clone()).collect::<Vec<_>>()
        }
    }
}

impl Trie {
    fn _print(&self, prefixes: Vec<String>, final_child: bool) {
        let chr = match self.data {
            None => "<root>".to_string(),
            Some(chr) => chr.to_string()
        };

        print!("{}{}, ({})\n", prefixes.join(""), chr, self.count);

        let new_prefix: Vec<String> = if prefixes.len() == 0 { 
            vec![] 
        } else {
            let mut a = prefixes[0..(prefixes.len() - 1)].to_vec();
            a.push(
                prefixes[prefixes.len() - 1]
                    .chars()
                    .filter(|&c| c == '\t')
                    .flat_map(|_| if final_child {
                            "\t".chars()
                        } else {
                            "│\t".chars()
                        }
                    )
                    .collect::<String>()
            );
            a
        };

        let mut i = 0;
        for child in &self.children {
            if i < self.children.len() - 1 {
                let mut child_prefix = new_prefix.clone();
                child_prefix.push("├──────\t".to_string());
                child._print(child_prefix, false);
            } else {
                let mut child_prefix = new_prefix.clone();
                child_prefix.push("└──────\t".to_string());
                child._print(child_prefix, true);
            }
            i += 1;
        }

    }

    fn print(&self) {
        self._print(vec!["".to_string()], true);
    }

}

fn string_to_trie(text: &str) -> Box<Trie> {
    let mut trie = Box::new(Trie {
        data: Some(text.chars().next().unwrap()),
        count: 1,
        children: vec![]
    });
    let reference = &trie;
    for c in text[1..].chars() {
        let c_trie = Box::new(Trie {
            data: Some(c),
            count: 1,
            children: vec![]
        });
        trie.children.push(c_trie);
    }
    return trie;
}

fn main() {
    let test_text = "test text";
    let t = Box::new(Trie {
        data: Some('t'),
        count: 0, 
        children: vec![]
    });
    let b = Box::new(Trie {
        data: Some('b'),
        count: 0, 
        children: vec![]
    });
    let a = Box::new(Trie {
        data: Some('a'),
        count: 0, 
        children: vec![t.clone(), b.clone()]
    });
    let s = Box::new(Trie {
        data: Some('s'),
        count: 0, 
        children: vec![t, a.clone(), b]
    });

    let c = Box::new(Trie {
        data: Some('c'),
        count: 0, 
        children: vec![s, a]
    });
    let trie = Box::new(Trie {
        data: None,
        count: 0, 
        children: vec![c]
    });

    trie.print();


    string_to_trie(test_text).print();

}




