{ 
pkgs ? import <nixpkgs> {},
}:
let 
	vkfft = import ./VkFFT.nix;
in
pkgs.mkShell {
	nativeBuildInputs = with pkgs.buildPackages; [ vkfft ];
}
