mod cl;
mod test_parser;

fn main() {
    cl::test_query();
    test_parser::test_configReader();
}
