fn main() {
    #[cfg(feature = "esp-target")]
    {
        embuild::espidf::sysenv::output();
    }
}
