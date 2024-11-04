pub fn decode_versions(versions: &str) -> Vec<String> {
    let mut sepcount = 0;
    let mut vertab: Vec<Vec<&str>> = versions
        .lines()
        .filter(|l| {
            if l.starts_with("===") {
                sepcount += 1;
                false
            } else {
                sepcount > 1 && sepcount < 3
            }
        })
        .map(|l| l.split(" ")
            .filter(|x| !x.trim().is_empty())
            .collect::<Vec<_>>())
        .filter(|v| !v.is_empty())
        .collect();

    let mut vervec = Vec::new();
    'zz: loop {
        for v in &mut vertab {
            if v.is_empty() { break 'zz; }
            vervec.push(v.remove(0).to_string());
        }
    }
    vervec
}

pub fn decode_java_versions(versions: &str) -> Vec<String> {
    let mut dash_lines = 0;
    let mut eq_lines = 0;

    let mut vertab: Vec<Vec<&str>> = versions
        .lines()
        .filter(|l| {
            if l.starts_with("---") {
                dash_lines += 1;
                false
            } else if l.starts_with("===") {
                eq_lines += 1;
                false
            } else  {
                dash_lines == 1 && eq_lines < 3
            }
        })
        .map(|l| l.split("|")
            .map(|x| x.trim())
            .collect::<Vec<_>>())
        .filter(|v| !v.is_empty())
        .collect();

    let mut vervec = Vec::new();
    for v in &mut vertab {
        vervec.push(v[5].to_string());
    }
    vervec
}
