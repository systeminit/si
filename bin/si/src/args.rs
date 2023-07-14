use clap::{builder::PossibleValuesParser, Parser, Subcommand};
use std::str::FromStr;
use strum::{Display, EnumString, EnumVariantNames};

const NAME: &str = "si";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative Launcher - takes the McFuckery out of DevOps.
#[derive(Debug, Parser)]
#[command(
name = NAME,
about = "The System Initiative Launcher - takes the McFuckery out of DevOps

                                              @@@@@@@@@                                             
                                  %@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@%                                 
                            @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@&                           
                       ,@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@                       
                    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@                   
                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@                
              /@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@              
            @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@&           
          @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@         
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@,       
       @@@@@@@@@@@@@@@@@                                                     @@@@@@@@@@@@@@@@@      
     /@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@     
    %@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@/   
   @@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@#  
  ,@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@  
  @@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@ 
 @@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@&
 @@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@                                  @@@@@@@@@@@@@@@@@@@@@@@
 @@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@@
@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@@
@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@@
@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@@
 @@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@@
 @@@@@@@@@@@@@@@@@@@@@@@                                    @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@@
 @@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@*
  @@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@@ 
  ,@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@@  
   #@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@@*  
    /@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@@    
      @@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@@*  @@@@@@@@@@@@@@@@@@     
       @@@@@@@@@@@@@@@@@                                                     @@@@@@@@@@@@@@@@@      
        ,@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@        
          @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@%         
            @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@#           
               @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@              
                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@                
                    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@%                   
                        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@                       
                            %@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*                           
                                   @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
")]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Checks that the system is setup correctly to run System Initiative
    Check(CheckArgs),
    /// Installs the necessary components to run System Initiative
    Install(InstallArgs),
    /// Launch the System Initiative Web UI.
    Launch(LaunchArgs),
}

#[derive(Debug, clap::Args)]
pub(crate) struct LaunchArgs {
    /// The System Initiative installation to launch
    #[arg(value_parser = PossibleValuesParser::new(Mode::variants()))]
    #[clap(short, long)]
    #[clap(default_value = "local")]
    mode: String,
}

#[derive(Debug, clap::Args)]
pub(crate) struct CheckArgs {}

#[derive(Debug, clap::Args)]
pub(crate) struct InstallArgs {
    /// The installation mode.
    #[arg(value_parser = PossibleValuesParser::new(Mode::variants()))]
    #[clap(short, long)]
    mode: String,
    /// Skip the system check as part of the install command
    #[clap(short, long)]
    pub skip_check: bool,
}

impl InstallArgs {
    pub(crate) fn mode(&self) -> Mode {
        Mode::from_str(&self.mode).expect("mode is a validated input str")
    }
}

impl LaunchArgs {
    pub(crate) fn mode(&self) -> Mode {
        Mode::from_str(&self.mode).expect("mode is a validated input str")
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString, EnumVariantNames)]
pub enum Mode {
    #[strum(serialize = "local")]
    Local,
}

impl Mode {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <Self as strum::VariantNames>::VARIANTS
    }
}
