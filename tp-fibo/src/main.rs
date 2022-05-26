//! This module implements fibonnaci sequence
use clap::Parser;

///Implemente parsers functionnalities
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]

///Args structure to implement flags options and input value
struct Args {
    #[clap(short, long)]

    verbose: bool, //type for flags is bool
    value: u32, //type for <VALUE> is u32

    #[clap(short='m', long="min", value_name="NUMBER")] //min value of the sequence
    min: Option<u32>, //min value of sequence 
}

///Main function prints fibonnaci terms which are available
fn main() {
    let args = Args::parse(); //import structure Args

    let min : u32; //min value must be sent as an u32

    let verbose = args.verbose;

    match args.min {
        Some(m) => min = m, 
        None => min = 0, //if user doesn't set a min the default value is 0
    }

    let value = args.value; //value is max term of fibonnaci user has sent

    if verbose { //if verbose true intermediar values are printed
        for i in min..=value{
            if fibo(i)==None {
                continue; //None are not printed
            }
            else {
                println!("fibo({:?}) = {:?}",i,fibo(i)) 
            }
        }
    } else { //if verbose is false only the last value (if existing) is returned
        if fibo(value) == None {
            println!("Result could not be calculated due to an overflow")
        } else {
            println!("fibo({:?}) = {:?}",value,fibo(value)) 
        }
    }
}

///This function returns the fibonacci sum or None as output for an u32 input 
fn fibo(n: u32) -> Option<u32> {

    //Declaration of variables
    let mut x1: u32 = 0;
    let mut x2: u32 = 1;
    let mut v: Option<u32> = None; //variable which contains either None or result of fibonnaci 

    if n == 0 {
        v = Some(0); //case 0 is set apart
    }

    else {
        for _ in 0..n { //addition is made n times

            match u32::checked_add(x1,x2) { //type of checked_add is None if an overflow occurs or u32
                Some(x) => {v = Some(x); x2 = x1; x1 = x}, //if there is a result v x2 and x1 are updated
                None => v = None //if there is an overflow v is set to None 
            }
        }
    }
    v //return value is v which is Option<u32> type
}
