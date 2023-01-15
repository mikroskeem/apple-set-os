{ lib
, naersk
, rev ? "dirty"
}:

naersk.buildPackage {
  pname = "apple-set-os";
  version = rev;

  src = ./.;
  cargoHash = "sha256-quCJn5VtZPZ7pkV+lDqkorsN3OrKebo8TGmREea4sjU=";

  cargoBuildOptions = x: x ++ [ "--target" "x86_64-unknown-uefi" ];
  doCheck = false; # not now
}
