use std::env;
use std::process::Command;
use std::str;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if enough arguments are provided
    if args.len() != 2 {
        eprintln!("Usage: {} <id>", args[0]);
        std::process::exit(1);
    }

    // Extract id from command-line arguments
    let id = &args[1];

    let command = "/firecracker-data/start.sh ".to_owned()  + id;
    
    // Spawn the shell process
    let status = Command::new("sudo")
        .arg("bash")
        .arg("-c")
        .arg(command)
        .status()
        .expect("Failed to start shell process");
     if status.success() {
        println!("Command executed successfully!");
    } else {
        println!("Command failed with {:?}", status);
    }

    // Get the IP address of the specified interface in the specified namespace
    let ip_address = get_interface_ip(id)
        .expect("Failed to get interface IP address");

    println!("IP Address: {}", ip_address);

    // Ping the obtained IP address
    let ping_result = ping_ip_address(&ip_address, id);
    match ping_result {
        Ok(output) => println!("Ping Result:\n{}", output),
        Err(err) => eprintln!("Failed to ping IP address: {}", err),
    }

}

fn get_interface_ip(id: &str) -> Result<String, String> {
    // Modify the command to use the provided id
    let output = Command::new("sudo")
        .args(&["ip", "-n", &format!("jailer-{}", id), "addr", "show", &format!("fc-{}-tap0", id)])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let output_str = str::from_utf8(&output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))?;

        let ip_address = parse_ip_from_ip_command_output(output_str)?;
        Ok(ip_address)
    } else {
        Err(format!("Command failed with {:?}", output.status))
    }

}

fn parse_ip_from_ip_command_output(output: &str) -> Result<String, String> {
    let mut ip_w_cidr = None;

    for line in output.lines() {
        if let Some(index) = line.find("inet ") {
            let fields: Vec<&str> = line[index..].split_whitespace().collect();
            if let Some(ip) = fields.get(1) {
                ip_w_cidr = Some(ip.to_string());
                break;
            }
        }
    }

    let ip_w_cidr = ip_w_cidr.ok_or_else(|| "IP address not found".to_string())?;

    // ip_w_cidr in the form 0.0.0.0/0 for the network namespaced side
    // of the tap device, we need to get the other side (-1) and remove the CIDR
    // block from the response
    let parts: Vec<&str> = ip_w_cidr.split('/').collect();
    let ip = parts.get(0).ok_or_else(|| "IP address not found".to_string())?.to_string();

    // Collect the last octet and subtract one
    let octets: Vec<&str> = ip.split('.').collect();
    let final_value = octets.last().ok_or_else(|| format!("Final value in IP range can't be found {:?}", octets.last()))?;
    let final_value = final_value.parse::<u8>().map_err(|_| "Final value is not a valid integer".to_string())?;

    // Subtract one from the final_value
    let final_value = final_value.wrapping_sub(1);

    // Create the final result string
    let mut result = String::from(&ip[..ip.rfind('.').unwrap_or(0) + 1]); // Extract the first three octets
    result.push_str(&final_value.to_string());

    Ok(result)
}

fn ping_ip_address(ip_address: &str, id: &str) -> Result<String, String> {
    let output = Command::new("sudo")
        .args(&["ip", "netns", "exec", &format!("jailer-{}", id), "ping", "-c", "1", ip_address])
        .output()
        .map_err(|e| format!("Failed to execute ping command: {}", e))?;

    if output.status.success() {
        let output_str = str::from_utf8(&output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))?;
        Ok(output_str.to_string())
    } else {
        Err(format!("Ping command failed with {:?}", output.status))
    }
}