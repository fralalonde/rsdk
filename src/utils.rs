use std::fs;
use std::io;
use std::error::Error;



/// Check if the given candidate is present in the list of available candidates.
pub fn test_candidate_present(candidate: &str) -> Result<(), Box<dyn Error>> {
    let candidates = get_available_candidates()?;

    if candidates.contains(&candidate.to_string()) {
        Ok(())
    } else {
        Err(format!("Stop! {} is not a valid candidate!", candidate).into())
    }
}

/// Retrieve the list of available candidates from a configuration file
fn get_available_candidates() -> Result<Vec<String>, io::Error> {
    let candidates_file = "~/.rsdk/candidates"; // Adjust path as necessary
    let contents = fs::read_to_string(candidates_file)?;

    // Split lines and filter out any empty lines
    let candidates = contents
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(candidates)
}

