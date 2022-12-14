const ipv4 = '(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)(?:\\.(?:25[0-5]|2[0-4]\\d|1\\d\\d|[1-9]\\d|\\d)){3}';
const v6segment = '[a-fA-F\\d]{1,4}';
const ipv6 = `
(?:
(?:${v6segment}:){7}(?:${v6segment}|:)|                                    // 1:2:3:4:5:6:7::  1:2:3:4:5:6:7:8
(?:${v6segment}:){6}(?:${ipv4}|:${v6segment}|:)|                             // 1:2:3:4:5:6::    1:2:3:4:5:6::8   1:2:3:4:5:6::8  1:2:3:4:5:6::1.2.3.4
(?:${v6segment}:){5}(?::${ipv4}|(?::${v6segment}){1,2}|:)|                   // 1:2:3:4:5::      1:2:3:4:5::7:8   1:2:3:4:5::8    1:2:3:4:5::7:1.2.3.4
(?:${v6segment}:){4}(?:(?::${v6segment}){0,1}:${ipv4}|(?::${v6segment}){1,3}|:)| // 1:2:3:4::        1:2:3:4::6:7:8   1:2:3:4::8      1:2:3:4::6:7:1.2.3.4
(?:${v6segment}:){3}(?:(?::${v6segment}){0,2}:${ipv4}|(?::${v6segment}){1,4}|:)| // 1:2:3::          1:2:3::5:6:7:8   1:2:3::8        1:2:3::5:6:7:1.2.3.4
(?:${v6segment}:){2}(?:(?::${v6segment}){0,3}:${ipv4}|(?::${v6segment}){1,5}|:)| // 1:2::            1:2::4:5:6:7:8   1:2::8          1:2::4:5:6:7:1.2.3.4
(?:${v6segment}:){1}(?:(?::${v6segment}){0,4}:${ipv4}|(?::${v6segment}){1,6}|:)| // 1::              1::3:4:5:6:7:8   1::8            1::3:4:5:6:7:1.2.3.4
(?::(?:(?::${v6segment}){0,5}:${ipv4}|(?::${v6segment}){1,7}|:))             // ::2:3:4:5:6:7:8  ::2:3:4:5:6:7:8  ::8             ::1.2.3.4
)(?:%[0-9a-zA-Z]{1,})?                                             // %eth0            %1
`.replace(/\s*\/\/.*$/gm, '').replace(/\n/g, '').trim();

const cidrv4 = `${ipv4}\\/(3[0-2]|[12]?[0-9])`;
const cidrv6 = `${ipv6}\\/(12[0-8]|1[01][0-9]|[1-9]?[0-9])`;

const cidr = `(?:^${cidrv4}$)|(?:^${cidrv6}$)`;

const cidrRegex = new RegExp(cidr);

const isCidr = (cidr) => cidrRegex.test(cidr);

function isValidCIDR(input) {
    const cidr = input.value;
    const valid = isCidr(cidr);
    return {
        valid,
        message: valid ? undefined : `'${cidr}' is not a valid IPV4 or IPV6 CIDR range`,
    };
}
