name: Build For Android
run-name: ${{ github.actor }} is building for Android
on: [push]
jobs:
  Build-Android:
    runs-on: ubuntu-latest
    environment: Android
    env:
      JAVA_HOME: /usr/lib/jvm/java-17-openjdk-amd64
      ANDROID_RELEASE_STORE_PASSWORD: ${{ secrets.ANDROID_RELEASE_STORE_PASSWORD }}
      ANDROID_RELEASE_KEY_PASSWORD: ${{ secrets.ANDROID_RELEASE_KEY_PASSWORD }}
      ANDROID_RELEASE_KEY_PATH: ${{ vars.ANDROID_RELEASE_KEY_PATH }}
      CARGO_TERM_COLOR: always
    steps:
      # Setup Build Environment
      - name: 🎉 The job was automatically triggered by a ${{ github.event_name }} event.
        run: echo "🎉 The job was automatically triggered by a ${{ github.event_name }} event."
      - name: 🐧 This job is now running on a ${{ runner.os }} server hosted by GitHub!
        run: echo "🐧 This job is now running on a ${{ runner.os }} server hosted by GitHub!"
      - name: 🔎 The name of your branch is ${{ github.ref }} and your repository is ${{ github.repository }}.
        run: echo "🔎 The name of your branch is ${{ github.ref }} and your repository is ${{ github.repository }}."
      - name: Check out repository code
        uses: actions/checkout@v3
        with:
          submodules: recursive
      - name: 💡 The ${{ github.repository }} repository has been cloned to the runner.
        run: echo "💡 The ${{ github.repository }} repository has been cloned to the runner."

      # List Files
      - name: List files in the repository
        run: ls ${{ github.workspace }}

      # Install Dependencies
      - name: Update APT Package Manager
        run: sudo apt update
      - name: Install APT Packages
        run: sudo apt -y install unzip openjdk-17-jre-headless

      # Extract Keystore
      - name: Extract Keystore
        run: echo "${{ secrets.RELEASE_KEY }}" | base64 -d > ${{ vars.ANDROID_RELEASE_KEY_PATH }}

      # Install Rust
      - name: Make Tools Directory
        run: mkdir -p ${{ github.workspace }}/tools
      - name: Download Rust Installer
        run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > ${{ github.workspace }}/tools/rust.sh
      - name: Make Rust Installer Executable
        run: chmod +x ${{ github.workspace }}/tools/rust.sh
      - name: Install Rust
        run: ${{ github.workspace }}/tools/rust.sh -y
      - name: Load Cargo Environment
        run: source "$HOME/.cargo/env"

      # Add Build Targets
      - name: Add armv7 Android Build Target
        run: $HOME/.cargo/bin/rustup target add armv7-linux-androideabi
      - name: Add aarch64 Android Build Target
        run: $HOME/.cargo/bin/rustup target add aarch64-linux-android
      - name: Add i686 Android Build Target
        run: $HOME/.cargo/bin/rustup target add i686-linux-android
      - name: Add x86_64 Android Build Target
        run: $HOME/.cargo/bin/rustup target add x86_64-linux-android

      # Build Native Libraries
      - name: Build Native Libraries (Release)
        working-directory: android
        run: ${{ github.workspace }}/android/gradlew externalNativeBuildRelease

      # Create Symlink for NDK
      - name: Create Symlink For NDK
        working-directory: /usr/local/lib/android/sdk/ndk
        run: |
          ls -liallh /usr/local/lib/android/sdk/ndk
          cd /usr/local/lib/android/sdk/ndk/`ls -1v /usr/local/lib/android/sdk/ndk | tail -n1`
          ln -s `pwd` ../current

      # Create config.toml File
      - name: Create config.toml File
        run: |
          export DESIRED_API=`grep "compileSdkVersion " ${{ github.workspace }}/android/app/build.gradle | rev | cut -d' ' -f1 | rev`
          sed "s/{android-version}/$DESIRED_API/g" ${{ github.workspace }}/.cargo/config.toml.sample > ${{ github.workspace }}/.cargo/config.toml.1
          sed "s:\$WORKSPACE:${{ github.workspace }}:g" ${{ github.workspace }}/.cargo/config.toml.1 > ${{ github.workspace }}/.cargo/config.toml

      # Build Engine As Library
      - name: Build CatgirlEngine For aarch64
        run: |
          ls -liallh ${{ github.workspace }}/android/app/build/intermediates/cmake/release/obj/arm64-v8a
          RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --verbose --target aarch64-linux-android --release --lib
      - name: Build CatgirlEngine For armv7
        run: |
          ls -liallh ${{ github.workspace }}/android/app/build/intermediates/cmake/release/obj/armeabi-v7a
          RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --verbose --target armv7-linux-androideabi --release --lib
      - name: Build CatgirlEngine For i686
        run: |
          ls -liallh ${{ github.workspace }}/android/app/build/intermediates/cmake/release/obj/x86
          RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --verbose --target i686-linux-android --release --lib
      - name: Build CatgirlEngine For x86_64
        run: |
          ls -liallh ${{ github.workspace }}/android/app/build/intermediates/cmake/release/obj/x86_64
          RUST_BACKTRACE=full $HOME/.cargo/bin/cargo build --verbose --target x86_64-linux-android --release --lib

      # Create Directories For Storing Engine in App
      - name: Create Directory To Store CatgirlEngine In App For aarch64
        run: mkdir -p ${{ github.workspace }}/android/app/src/main/jniLibs/arm64-v8a
      - name: Create Directory To Store CatgirlEngine In App For armv7
        run: mkdir -p ${{ github.workspace }}/android/app/src/main/jniLibs/armeabi-v7a
      - name: Create Directory To Store CatgirlEngine In App For i686
        run: mkdir -p ${{ github.workspace }}/android/app/src/main/jniLibs/x86
      - name: Create Directory To Store CatgirlEngine In App For x86_64
        run: mkdir -p ${{ github.workspace }}/android/app/src/main/jniLibs/x86_64

      # Copy Engine To App
      - name: Store CatgirlEngine In App For aarch64
        run: cp -av ${{ github.workspace }}/target/aarch64-linux-android/release/libmain.so ${{ github.workspace }}/android/app/src/main/jniLibs/arm64-v8a/libmain.so
      - name: Store CatgirlEngine In App For armv7
        run: cp -av ${{ github.workspace }}/target/armv7-linux-androideabi/release/libmain.so ${{ github.workspace }}/android/app/src/main/jniLibs/armeabi-v7a/libmain.so
      - name: Store CatgirlEngine In App For i686
        run: cp -av ${{ github.workspace }}/target/i686-linux-android/release/libmain.so ${{ github.workspace }}/android/app/src/main/jniLibs/x86/libmain.so
      - name: Store CatgirlEngine In App For x86_64
        run: cp -av ${{ github.workspace }}/target/x86_64-linux-android/release/libmain.so ${{ github.workspace }}/android/app/src/main/jniLibs/x86_64/libmain.so

      # Compile Program
      - name: Build Android App as APK (Release)
        working-directory: android
        run: ${{ github.workspace }}/android/gradlew assembleRelease

      - name: Build Android App as Bundle (Release)
        working-directory: android
        run: ${{ github.workspace }}/android/gradlew bundleRelease

      # Display APK Directory
      - name: Display APK Directory (Release)
        run: ls -liallh ${{ github.workspace }}/android/app/build/outputs/apk/release

      # Display Bundle Directory
      - name: Display Bundle Directory (Release)
        run: ls -liallh ${{ github.workspace }}/android/app/build/outputs/bundle/release

      # Prepare Artifact Uploads
      - name: Prepare Artifact Uploads
        run: |
          mkdir -p ${{ github.workspace }}/upload
          mv ${{ github.workspace }}/android/app/build/outputs/apk/release/app-release.apk ${{ github.workspace }}/upload
          mv ${{ github.workspace }}/android/app/build/outputs/bundle/release/app-release.aab ${{ github.workspace }}/upload
          mv ${{ github.workspace }}/android/app/build/outputs/mapping/release/mapping.txt ${{ github.workspace }}/upload
          mv ${{ github.workspace }}/target/binding ${{ github.workspace }}/upload

      # Upload APK
      - name: Upload APK (Release)
        uses: actions/upload-artifact@v3
        with:
          name: CatgirlEngine-Android-APK
          path: |
            ${{ github.workspace }}/upload/app-release.apk
            ${{ github.workspace }}/upload/mapping.txt
            ${{ github.workspace }}/upload/binding

      # Upload Bundle
      - name: Upload Bundle (Release)
        uses: actions/upload-artifact@v3
        with:
          name: CatgirlEngine-Android-Bundle
          path: |
            ${{ github.workspace }}/upload/app-release.aab
            ${{ github.workspace }}/upload/mapping.txt
            ${{ github.workspace }}/upload/binding

      # TODO: Upload to Play Store (and Github Releases) On Reading "Publish" in Commit Message

      - name: List All Files
        run: find ${{ github.workspace }}
      - name: List All Installed Packages
        run: |
          apt list --installed | wc -l
          apt list --installed

      # Display Build Status
      - name: 🍏 This job's status is ${{ job.status }}.
        run: echo "🍏 This job's status is ${{ job.status }}."