# with import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/c79b98dd409766c342cb4917ed23f2c56b80f38d.tar.gz") {};
with import <nixpkgs> {};
stdenv.mkDerivation rec {
	pname = "VkFFT";
	version = "1.3.2";

	src = fetchFromGitHub {
	    owner = "DTolm";
	    repo = "VkFFT";
	    rev = "v${version}";
	    sha1 = "sha1-QPyE3aIWA62jYwTowABhoaxRCAo=";
	};

	nativeBuildInputs = [ cmake glslang shaderc.bin pkg-config ];
	propagatedBuildInputs = [vulkan-loader vulkan-headers];
	patches = [ ./vkfft_glslang.patch ];
	# buildInputs = [ pkg-config glslang shaderc.bin vulkan-loader vulkan-headers];
	# src = ./VkFFT/..;
	cmakeFlags = [
	# "-Dglslang_SOURCE_DIR=${glslang.src}"
	"-Dglslang_INCLUDE_DIR=${glslang}/include/glslang/Include"
	"-DCMAKE_CXX_FLAGS=-Wno-unused-result"
	# "-DCMAKE_VERBOSE_MAKEFILE=on"
	];
}
