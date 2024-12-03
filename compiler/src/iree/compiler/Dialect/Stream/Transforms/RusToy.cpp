extern "C" {
void cxxbridge1$say_hello() noexcept;
} // extern "C"

void say_hello() noexcept {
  cxxbridge1$say_hello();
}
