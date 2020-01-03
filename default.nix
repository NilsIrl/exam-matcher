with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "exam-matcher";
  LIBCLANG_PATH="${llvmPackages.libclang}/lib";
  buildInputs = [
    tesseract4
    leptonica
    poppler_utils
    clang
  ];
}
