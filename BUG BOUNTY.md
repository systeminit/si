# Bug Bounty at System Initiative

**Launch Date:** 27th November, 2024 | **Last updated:** 24th May, 2025  

Welcome to the **System Initiative Bug Bounty** guidelines. This document serves as a comprehensive reference for security researchers and stakeholders participating in our bug bounty program. It outlines our commitment to security, the rules of engagement, reward structures, and the scope of eligible and ineligible vulnerabilities.

Our goal is to foster collaboration with the security community to identify and resolve potential vulnerabilities in our systems. By following this guide, you can help ensure the safety and integrity of our platforms while being recognized and rewarded for your valuable contributions. 

All contributions endeavoring to maintain the safety of our platform are appreciated but not at all are eligble for reward.

The associated rules, qualifications, and rewards for this program may be revised at any time and any decisions made by security@systeminit.com will be considered final.

# Contained within this document:
- **Disclosure Guidelines**: The rules and guidelines for reporting vulnerabilities.
- **Rewards**: Information on bounty payouts based on severity.
- **Ineligible Findings**: Specific exclusions from the policy.
- **Scope**: Details about what assets are in and out of scope.

System Initiative's Bounty Standards set expectations for both System Initiative and hackers when assessing the reward for a report. The standards begin with a relevant rating system such as [CVSS](https://nvd.nist.gov/vuln-metrics/cvss) for report assessments and then additionally consider additional adjustments ("bumps") for discrepancies between associated rating scores and actual business impact. This ensures more accurate correlations between vulnerability impact evaluations to reward bounties.

These Platform Standards aim to ensure consistency, fairness, and the best results for all program participants. 

We value your expertise and collaboration in helping make System Initiative more secure for everyone. 

---

# Disclosure Guidelines

### Disclosure Philosophy

#### Hackers Should:
- Ethically respect the rules and privacy of others.
- Make a good-faith effort to support their reports.
- Act for the common good by reporting vulnerabilities promptly and responsibly.

#### Preferred Report Characteristics:
- Accurate information on how a vulnerability can be exploited.
- A reproducible proof-of-concept where one was not previously available. 

If either of these characteristics are not met, the report may still be rewarded under certain circumstances. Some examples of this situation would be:
- If validating the proof of concept will go against the bug bounty rules
- If some of the information in the report had to be guessed due to internal information that is not available to the reporter

Reports that restate information shared in recent blogs, existing reports, or news articles are less likely to be rewarded.

### Key Points
Please submit all reports to [security@systeminit.com](mailto:security@systeminit.com) with the following headings:
  - Executive Summary
    - Required details:
    - URL and affected parameters.
    - Browser, OS, or app version.
    - Perceived impact (e.g., exploitation potential).
  - Bug Impact.
  - Proof of Concept/Reproduction.
  - (optional) Possible Mitigation Steps.

Vulnerabilities are classified by **risk level** and **impact**:
- Data exposed.
- Privilege level obtained.
- Systems or users affected.

**Do not pivot further** into the network or alternative account/workspace after finding a vulnerability.
- Avoid testing third-party service providers or non-System Initiative assets.

**If you encounter Personally Identifiable Information (PII):**
  - Contact us immediately at [security@systeminit.com](mailto:security@systeminit.com).
  - Do not proceed with accessing or retaining local copies.

**Limits/Additional Considerations:**
- **One Vulnerability per Report**: Unless vulnerabilities need to be chained for impact.
- Limit automated scanning to **50 requests per second**.
- Aggressive testing causing service degradation or unnecessarily deep service penetration will result in permanent ineligibility from the program.
- Multiple reports of the same vulnerability class on similar endpoints will pay for the value received. However, reports that alert System Initiative to a systemic issue across its assets will be considered for a discretionary bonus.
- Reports demonstrating significant security issues through chained vulnerabilities will be evaluated based on overall impact. System Initiative will issue bonuses for distinct reports that comprise a serious vulnerability chain.
- Vulnerabilities allowing attackers to bypass encryption in client applications (e.g., Adversary In The Middle attacks) will be prioritized. Vulnerabilities requiring attackers to disable certificate pinning are not valid.
- Hackers must report vulnerabilities in third-party components to the component owner before disclosing them to System Initiative. System Initiative will reward valid reports for vulnerabilities in third-party components that lead to urgent fixes or drive value outside regular patching schedules or SLA's.
- Reports exposing sensitive PII for multiple users will be rated Critical. Sensitive PII includes Social Security numbers, passport numbers, driver’s license numbers, hashed passwords, and credit card numbers.
- System Initiative will reward valid reports of leaked credentials unknown to the program or monitoring tools. Hackers must include the leak's source and avoid testing beyond authenticating and deauthenticating.
- A bypass of a resolved vulnerability will be treated as a new report. System Initiative will consider awarding bonuses for reports providing additional bypass information
- Reporters must not have: 
  - Conducted phishing, vishing, smashing, or any other active user misleading.
  - Accessed data not owned by the reporter.
  - Interacted with accounts the reporter does not have explicit permission to access.
- Bug reports must not be submitted from any country sanctioned by the United States. At the time of writing, this includes, but is not limited to, China, Cuba, the Islamic Republic Of Iran, the Democratic People's Republic Of Korea, the Russian Federation, Sudan, Syrian Arab Republic.

### Paying for New Zero Days

Reports for vulnerabilities that were either recently disclosed to the public or where a patch was recently released may not be eligible for a reward. "Recently" can be assumed to be 90 calendar days before the current point in time. This is because **System Initiative** already tracks many of these announcements and needs an appropriate amount of time to test and implement patches and safeguards before a report with the same information can provide value. However, each submission is evaluated on a case-by-case basis, and those providing significant value or new information such as more simplistic mitigation steps may still be eligible for a reward.

---

# Rewards

**System Initiative** rewards findings based on **security impact** and **CVSS 3.0 base score**. These values are guidelines only and depend on both the quality of the report and the potential impact

### Severity Levels
1. **Critical Impact**
   - Examples: RCE, SQL Injection
   - Reward: Up to $2,500 USD (based on impact).

2. **High Impact**
   - Examples: SSRF, XXE, significant authentication bypasses.
   - Reward: Up to $1,500 USD.

3. **Medium Impact**
   - Examples: Stored XSS, memory leaks, IDORs.
   - Reward: Up to $500 USD.

4. **Low Impact**
   - Examples: Reflected XSS, open redirects.
   - Reward: Standard $50–$250 USD.

5. **Informational Issues**
   - These are not eligible for rewards but may receive a goodwill $50 USD gesture or a piece of highly sought after System Initiatve swag. 

# Core Ineligible Findings

When reporting potential vulnerabilities, please consider the following:  
1. **Realistic attack scenarios**  
2. **The security impact of the behavior**  

# Ineligible Findings:
- **Theoretical Vulnerabilities**: Issues requiring unlikely user interaction or conditions, such as unsupported browsers or devices.
- **Vulnerabilities Without Real-World Impact**: Clickjacking on non-sensitive pages, permissive CORS configurations without demonstrated risk.
- **Optional Security Hardening**: SSL/TLS configurations, missing best practices (e.g., cookie flags, SPF/DKIM).
- **Hazardous Testing**: DoS attacks, spamming, social engineering, or any testing that affects system availability.


Below, you will find the most common false positives we encounter. The following issues will be closed as invalid except in rare circumstances demonstrating clear security impact either individually or as part of a vulnerable chain.

## Theoretical Vulnerabilities Requiring Unlikely User Interaction or Circumstances  

Examples include:  
- Vulnerabilities affecting unsupported or end-of-life browsers or operating systems  
- Broken link hijacking  
- Tabnabbing  
- Content spoofing and text injection issues  
- Attacks requiring physical access to a device (unless explicitly in scope)  
- Self-exploitation, such as self-XSS or self-DoS (unless it can be used to attack a different account)  

## Theoretical Vulnerabilities Without Real-World Security Impact  

Examples include:  
- Clickjacking on pages with no sensitive actions  
- Cross-Site Request Forgery (CSRF) on forms with no sensitive actions (e.g., Logout)  
- Permissive CORS configurations without demonstrated security impact  
- Software version disclosure / Banner identification issues / Descriptive error messages or headers (e.g., stack traces, application or server errors)  
- Comma Separated Values (CSV) injection  
- Open redirects (unless you can demonstrate additional security impact)  

## Optional Security Hardening Steps / Missing Best Practices  

Examples include:  
- SSL/TLS Configurations  
- Lack of SSL Pinning  
- Lack of jailbreak detection in mobile apps  
- Cookie handling (e.g., missing HttpOnly/Secure flags)  
- Content-Security-Policy configuration opinions  
- Optional email security features (e.g., SPF/DKIM/DMARC configurations)  
- Most issues related to rate limiting  

## Vulnerabilities Requiring Hazardous Testing  

This type of testing must **never** be attempted unless explicitly authorized. Examples include:  
- Issues relating to excessive traffic/requests (e.g., DoS, DDoS)  
- Any other issues where testing may affect the availability of systems  
- Social engineering attacks (e.g., phishing, opening support requests)  
- Attacks that are noisy to users or admins (e.g., spamming notifications or forms)  
- Attacks against physical facilities  

---

# Scope

### Eligible Assets

| Identifier                      | Asset Type         | Instruction | Eligible for Bounty | Eligible for Submission | Availability Requirement | Confidentiality Requirement | Integrity Requirement | Max Severity | System Tags  | Created At                 | Updated At                 |
|---------------------------------|--------------------|-------------|---------------------|--------------------------|--------------------------|-----------------------------|-----------------------|--------------|--------------|---------------------------|---------------------------|
| app.systeminit.com              | WEB APP           |             | true                | true                     |                          |                             |                       | critical     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| app.systeminit.com/api          | API               |             | true                | true                     |                          |                             |                       | critical     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| api.systeminit.com          | API                   |             | true                | true                     |                          |                             |                       | critical     | production   | 2025-05-24 21:00:00 UTC    | 2025-05-24 21:00:00 UTC    |
| auth-api.systeminit.com         | API               |             | true                | true                     |                          |                             |                       | critical     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| e.systeminit.com                | DOMAIN REFERENCE  |             | true                | true                     |                          |                             |                       | medium     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| module-index.systeminit.com     | API               |             | true                | true                     |                          |                             |                       | critical     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| systeminit.com                  | WEB APP           |             | true                | true                     |                          |                             |                       | medium     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| tools.systeminit.com            | WEB APP           |             | true                | true                     |                          |                             |                       | critical     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| tools.systeminit.com/api        | API               |             | true                | true                     |                          |                             |                       | critical     | production   | 2024-11-28 13:45:00 UTC    | 2024-11-28 13:45:00 UTC    |
| api.tools.systeminit.com    | API               |             | true                | true                     |                          |                             |                       | critical     | production   | 2025-05-24 21:00:00 UTC    | 2025-05-24 21:00:00 UTC    |

