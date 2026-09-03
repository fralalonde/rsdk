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
        .map(|l| {
            l.split(" ")
                .filter(|x| !x.trim().is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .collect();

    let mut vervec = Vec::new();
    'vv: loop {
        for v in &mut vertab {
            if v.is_empty() {
                break 'vv;
            }
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
            } else {
                dash_lines == 1 && eq_lines < 3
            }
        })
        .map(|l| l.split("|").map(|x| x.trim()).collect::<Vec<_>>())
        .filter(|v| !v.is_empty())
        .collect();

    let mut vervec = Vec::new();
    for v in &mut vertab {
        // The identifier lives in the last column of the pipe-delimited table.
        // Read from the end (rather than a fixed index) so a change in the
        // number of leading columns doesn't panic.
        if let Some(id) = v.last() {
            let id = id.trim();
            if !id.is_empty() {
                vervec.push(id.to_string());
            }
        }
    }
    vervec
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mirrors the current SDKMAN API response shape: a `===`-framed header,
    // a `---`-separated 4-column table, and a `===`-framed footer.
    const NEW_FORMAT: &str = "\
================================================================================
Available Java Versions for Cygwin
================================================================================
 Vendor         | Use | Version            | Identifier
--------------------------------------------------------------------------------
 Corretto       |     | 26.0.2             | 26.0.2-amzn
                |     | 25.0.4             | 25.0.4-amzn
 Temurin        |     | 21.0.4             | 21.0.4-tem
================================================================================
 > in use   * installed   + local only
--------------------------------------------------------------------------------
 $ sdk install java <Identifier>    install a specific version
================================================================================";

    #[test]
    fn decodes_identifier_from_last_column() {
        let versions = decode_java_versions(NEW_FORMAT);
        assert_eq!(versions, vec!["26.0.2-amzn", "25.0.4-amzn", "21.0.4-tem"]);
    }

    #[test]
    fn skips_short_or_empty_rows_without_panicking() {
        let input = "\
================================================================================
Available Java Versions
================================================================================
 Vendor | Use | Version | Identifier
--------------------------------------------------------------------------------
 only-three | fields | here
================================================================================";
        // The 3-field row's last field is used; no out-of-bounds panic.
        let versions = decode_java_versions(input);
        assert_eq!(versions, vec!["here"]);
    }
}
