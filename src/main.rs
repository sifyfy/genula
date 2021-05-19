use anyhow::anyhow;
use mac_address::MacAddress;
use sha1::{Digest, Sha1};
use std::net::ToSocketAddrs;


fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();

    let mac_address = if opts.use_mac_address_of_this_node {
        let mac_address = mac_address::get_mac_address()?
            .ok_or(anyhow!("no MAC Address. You have to use --mac-address CLI parameter to tell MAC address to me by manually."))?;
        Some(mac_address)
    } else {
        opts.mac_address
    };

    // optsに既に必要な情報が入っている場合はGUIを出さずに結果を返す。
    match (
        mac_address,
        opts.unique_identifier.as_ref().map(|s| s.as_str()),
    ) {
        (None, None) => {
            // GUIモードへ
            todo!("*** Sorry, GUI mode is not yet implemented. See the command help (run with `--help` flag) and use CLI mode. ***")
        }
        a => {
            // CUIモードへ
            let ntp_server_addr = format!("{}:{}", opts.ntp_server, opts.ntp_server_port);
            let result = match a {
                (Some(mac_address), _) => mac_address.generate_ula_prefix(ntp_server_addr),
                (_, Some(unique_identifier)) => {
                    unique_identifier.generate_ula_prefix(ntp_server_addr)
                }
                (None, None) => panic!(), // GUIモードに遷移していてここには来ないはず
            };
            match result {
                Ok(ula_prefix) => println!("ULA prefix: {}", ula_prefix.to_string()),
                Err(err) => eprintln!("Error: {}", err),
            }
        }
    }

    Ok(())
}

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opts {
    #[structopt(long, default_value = "0.pool.ntp.org")]
    ntp_server: String,

    #[structopt(long, default_value = "123")]
    ntp_server_port: u16,

    /// The MAC address of this node is used automatically.
    /// When this flag is set, `--mac-address` CLI parameter is ignored.
    #[structopt(long)]
    use_mac_address_of_this_node: bool,

    #[structopt(long, parse(try_from_str))]
    mac_address: Option<MacAddress>,

    #[structopt(long)]
    unique_identifier: Option<String>,
}

impl Opts {
    fn from_args() -> Self {
        <Self as structopt::StructOpt>::from_args()
    }
}

struct UlaPrefix([u8; 8]);

impl UlaPrefix {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .enumerate()
            .map(|(i, byte)| {
                if i % 2 == 0 || i == self.0.len() - 1 {
                    format!("{:02x}", byte)
                } else {
                    format!("{:02x}:", byte)
                }
            })
            .collect()
    }
}

trait GenerateUlaPrefix {
    type Error;
    fn generate_ula_prefix(
        &self,
        ntp_server_addr: impl ToSocketAddrs,
    ) -> Result<UlaPrefix, Self::Error>;
}

impl GenerateUlaPrefix for MacAddress {
    type Error = ntp::errors::Error;

    fn generate_ula_prefix(
        &self,
        ntp_server_addr: impl ToSocketAddrs,
    ) -> Result<UlaPrefix, Self::Error> {
        let eui64 = {
            let bytes = self.bytes();
            [
                bytes[0] | 0b00000010 & 0b11111110,
                bytes[1],
                bytes[2],
                0xff,
                0xfe,
                bytes[3],
                bytes[4],
                bytes[5],
            ]
        };
        generate_ula_prefix(&eui64, ntp_server_addr)
    }
}

impl GenerateUlaPrefix for &'_ str {
    type Error = ntp::errors::Error;

    fn generate_ula_prefix(
        &self,
        ntp_server_addr: impl ToSocketAddrs,
    ) -> Result<UlaPrefix, Self::Error> {
        generate_ula_prefix(self.as_bytes(), ntp_server_addr)
    }
}

fn generate_ula_prefix(
    identifier: &[u8],
    addr: impl ToSocketAddrs,
) -> Result<UlaPrefix, ntp::errors::Error> {
    let ntp_time_bytes = get_ntp_time(addr)?;
    let bytes: Vec<u8> = ntp_time_bytes
        .iter()
        .chain(identifier)
        .map(Clone::clone)
        .collect();
    match Sha1::digest(&bytes).as_slice() {
        &[.., b1, b2, b3, b4, b5] => Ok(UlaPrefix([0xfd, b1, b2, b3, b4, b5, 0x00, 0x00])),
        _ => panic!(),
    }
}

fn get_ntp_time(addr: impl ToSocketAddrs) -> Result<[u8; 8], ntp::errors::Error> {
    let response = ntp::request(addr)?;
    let sec_bytes = response.transmit_time.sec.to_be_bytes();
    let frac_bytes = response.transmit_time.frac.to_be_bytes();
    Ok([
        sec_bytes[0],
        sec_bytes[1],
        sec_bytes[2],
        sec_bytes[3],
        frac_bytes[0],
        frac_bytes[1],
        frac_bytes[2],
        frac_bytes[3],
    ])
}
