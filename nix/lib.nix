inputs:
{ pkgs }:
let
  inherit (pkgs.lib) concatStringsSep;
  inherit (pkgs.stdenv.hostPlatform) system;

  platform =
    {
      aarch64-darwin = "darwin";
      x86_64-darwin = "darwin";
      x86_64-linux = "linux";
      aarch64-linux = "linux";
    }
    .${system};

  useLinker =
    linker:
    concatStringsSep " " [
      "-C"
      "linker=${pkgs.clang}/bin/clang"
      "-C"
      "link-arg=--ld-path=${linker}"
    ];

  RUSTFLAGS =
    {
      darwin = useLinker "${pkgs.lld}/bin/ld64.lld";
      linux = useLinker "${pkgs.mold}/bin/mold";
    }
    ."${platform}";

  root = ./..;

  rust = pkgs.rust-bin.fromRustupToolchainFile "${root}/rust-toolchain.toml";

  # --- Android / Tauri mobile toolchain --------------------------------------

  androidTargets = [
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "i686-linux-android"
    "x86_64-linux-android"
  ];

  rustToolchainFile = builtins.fromTOML (builtins.readFile "${root}/rust-toolchain.toml");

  # Same toolchain as `rust`, but with the Android target std libraries bundled
  # in so `cargo build --target <android-triple>` works without rustup (which is
  # not present in the Nix shells).
  androidRust = pkgs.rust-bin.fromRustupToolchain (
    rustToolchainFile.toolchain
    // {
      targets = (rustToolchainFile.toolchain.targets or [ ]) ++ androidTargets;
    }
  );

  androidNdkVersion = "27.2.12479018";
  androidBuildToolsVersions = [
    "34.0.0"
    "35.0.0"
    "36.0.0"
  ];
  # Tauri's generated Android project currently targets `compileSdk = 36`;
  # the older ones are kept so a `compileSdk` bump/downgrade keeps working.
  androidPlatformVersions = [
    "34"
    "35"
    "36"
  ];

  androidComposition = pkgs.androidenv.composeAndroidPackages {
    cmdLineToolsVersion = "19.0";
    platformToolsVersion = "36.0.2";
    buildToolsVersions = androidBuildToolsVersions;
    platformVersions = androidPlatformVersions;
    includeNDK = true;
    ndkVersions = [ androidNdkVersion ];
    includeEmulator = false;
    includeSystemImages = false;
  };

  androidSdkRoot = "${androidComposition.androidsdk}/libexec/android-sdk";
  androidNdkRoot = "${androidSdkRoot}/ndk/${androidNdkVersion}";
  androidAapt2 = "${androidSdkRoot}/build-tools/${pkgs.lib.last androidBuildToolsVersions}/aapt2";

  androidEnv = {
    JAVA_HOME = "${pkgs.jdk17.home}";
    ANDROID_HOME = androidSdkRoot;
    ANDROID_SDK_ROOT = androidSdkRoot;
    ANDROID_NDK_ROOT = androidNdkRoot;
    ANDROID_NDK_HOME = androidNdkRoot;
    NDK_HOME = androidNdkRoot;
    # The Android Gradle Plugin pulls a prebuilt Linux aapt2 from Maven that
    # isn't patched for NixOS; point it at the SDK's (autoPatchelf'd) one.
    GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidAapt2}";
  };
in
{
  inherit inputs;
  inherit rust root;
  inherit
    androidRust
    androidComposition
    androidEnv
    androidNdkVersion
    androidNdkRoot
    androidSdkRoot
    ;

  rustPlatform = pkgs.makeRustPlatform {
    rustc = rust;
    cargo = rust;
  };

  env = {
    inherit RUSTFLAGS;

    RUST_LOG = "trace";
    RUSTFMT = "${pkgs.rust-bin.nightly.latest}/bin/rustfmt";
  };

  cargoLock.lockFile = ../Cargo.lock;
}
