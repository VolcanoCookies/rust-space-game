use spacegame::Entity;
use spacegame_core::network_id::NetworkId;

fn main() {
    let a = 13172787500349055146u64;
    let b = 8290199278618935309u64;
    let c = 1812127914u64;

    println!("{:?}", c);
    println!("{:?}", b >> 32);
    println!("{:?}", a >> 32);

    println!("{:?}", c << 32);
    println!("{:?}", b >> 32);
    println!("{:?}", a >> 32);
}
