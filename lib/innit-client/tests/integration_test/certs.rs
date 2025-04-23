use rcgen::{
    BasicConstraints,
    Certificate,
    CertificateParams,
    DnType,
    DnValue::PrintableString,
    ExtendedKeyUsagePurpose,
    IsCa,
    KeyPair,
    KeyUsagePurpose,
};
use time::{
    Duration,
    OffsetDateTime,
};

pub fn new_ca() -> (Certificate, KeyPair) {
    let mut params =
        CertificateParams::new(Vec::default()).expect("empty subject alt name can't produce error");
    let (yesterday, tomorrow) = validity_period();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.distinguished_name.push(
        DnType::CountryName,
        PrintableString("BR".try_into().expect("should parse")),
    );
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Crab widgits SE");
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);

    params.not_before = yesterday;
    params.not_after = tomorrow;

    let key_pair = KeyPair::generate().expect("should generate key pair");
    (
        params.self_signed(&key_pair).expect("should sign"),
        key_pair,
    )
}

pub fn new_end_entity(ca: &Certificate, ca_key: &KeyPair) -> (Certificate, KeyPair) {
    let name = "entity.other.host";
    let mut params = CertificateParams::new(vec![name.into()]).expect("we know the name is valid");
    let (yesterday, tomorrow) = validity_period();
    params.distinguished_name.push(DnType::CommonName, name);
    params.use_authority_key_identifier_extension = true;
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);
    params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ClientAuth);
    params.not_before = yesterday;
    params.not_after = tomorrow;

    let key_pair = KeyPair::generate().expect("should generate key pair");
    (
        params
            .signed_by(&key_pair, ca, ca_key)
            .expect("should sign"),
        key_pair,
    )
}

pub fn validity_period() -> (OffsetDateTime, OffsetDateTime) {
    let day = Duration::new(86400, 0);
    let yesterday = OffsetDateTime::now_utc()
        .checked_sub(day)
        .expect("should be yesterday");
    let tomorrow = OffsetDateTime::now_utc()
        .checked_add(day)
        .expect("should be tomorrow");
    (yesterday, tomorrow)
}
