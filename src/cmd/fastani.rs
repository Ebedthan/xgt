use crate::utils::{email_check, FastAniArgs};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::time::{Duration, UNIX_EPOCH};
use ureq::Agent;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RequestBodyParameters {
    kmer: u8,
    frag_len: u16,
    min_frag: u8,
    min_frac: f32,
    version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RequestBody {
    query: Vec<String>,
    reference: Vec<String>,
    parameters: RequestBodyParameters,
    priority: String,
    email: String,
}

impl RequestBody {
    pub fn create_request_body(params: &FastAniArgs) -> Self {
        RequestBody {
            query: vec![params.query.as_ref().unwrap().to_string()],
            reference: vec![params.reference.as_ref().unwrap().to_string()],
            parameters: RequestBodyParameters {
                kmer: params.kmer,
                frag_len: params.frag_len,
                min_frag: params.min_frag,
                min_frac: params.min_frac,
                version: "1.33".to_string(),
            },
            priority: "secret".to_string(),
            email: params.email.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AniResult {
    query: String,
    reference: String,
    data: AniDataResult,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AniDataResult {
    ani: f32,
    af: f32,
    mapped: u16,
    total: u32,
    status: String,
    stdout: String,
    stderr: String,
    cmd: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FastAniResult {
    job_id: String,
    group_1: Vec<String>,
    group_2: Vec<String>,
    parameters: RequestBodyParameters,
    results: Vec<AniResult>,
    #[serde(alias = "positionInQueue")]
    position_in_queue: Option<u16>,
}

pub fn create_fastani_job(params: &FastAniArgs) -> Result<()> {
    let data = serde_json::to_value(&RequestBody::create_request_body(params))?;

    let response = match ureq::post("https://gtdb-api.ecogenomic.org/fastani")
        .set("accept", "application/json")
        .set("Content-Type", "application/json")
        .send_json(data)
    {
        Ok(r) => r,
        Err(ureq::Error::Status(400, _)) => {
            bail!("No comparisons could be made as one or more genomes were not found in the database.");
        }
        Err(_) => {
            bail!("There was an error making the request or receiving the response.");
        }
    };

    let fastani: FastAniResult = response.into_json()?;
    let fastani_string = serde_json::to_string_pretty(&fastani)?;

    let output = params.get_output();

    if let Some(path) = output {
        let path_clone = path.clone();
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .with_context(|| format!("Failed to create file {path_clone}"))?;
        file.write_all(fastani_string.as_bytes())
            .with_context(|| format!("Failed to write to {path_clone}"))?;
    } else {
        writeln!(io::stdout(), "{fastani_string}")?;
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FastAniJobInfo {
    #[serde(alias = "jobId")]
    job_id: String,
    #[serde(alias = "createdOn")]
    created_on: f64,
    status: String,
}

pub fn get_job_info(job_id: String) -> Result<()> {
    let agent: Agent = ureq::AgentBuilder::new().build();

    let response = match agent
        .get(format!("https://gtdb-api.ecogenomic.org/fastani/{}/info", job_id).as_ref())
        .set("accept", "application/json")
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::Status(code, _)) => {
            bail!("The server returned an unexpected status code ({})", code);
        }
        Err(_) => {
            bail!("There was an error making the request or receiving the response.");
        }
    };

    let fastani_job_info: FastAniJobInfo = response.into_json()?;

    let system_time = UNIX_EPOCH + Duration::from_secs_f64(fastani_job_info.created_on);
    let datetime = DateTime::<Utc>::from(system_time);
    let timestamp_str = datetime.format("%a, %d %b %G %T %Z").to_string();

    println!(
        "Job ({}) created on {} is {}",
        fastani_job_info.job_id, timestamp_str, fastani_job_info.status
    );

    Ok(())
}

pub fn get_job_result(job_id: String) -> Result<()> {
    let agent: Agent = ureq::AgentBuilder::new().build();

    let response = match agent
        .get(format!("https://gtdb-api.ecogenomic.org/fastani/{}", job_id).as_ref())
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::Status(code, _)) => {
            bail!("The server returned an unexpected status code ({})", code);
        }
        Err(_) => {
            bail!("There was an error making the request or receiving the response.");
        }
    };

    Ok(())
}
