sdkmanager "platforms;android-34" "system-images;android-34;google_apis;x86_64"

avdmanager create avd -n dioxus_device -k "system-images;android-34;google_apis;x86_64" -d pixel_6

emulator -avd dioxus_device -no-snapshot -no-boot-anim -wipe-data -accel on -gpu swiftshader_indirect -memory 1024 -cores 2


avdmanager create avd \
  -n dioxus_device \
  -k "system-images;android-34;google_apis;x86_64" \
  -d pixel_6

emulator -avd dioxus_device \
  -no-snapshot \
  -no-boot-anim \
  -wipe-data \
  -accel on \
  -gpu swiftshader_indirect \
  -memory 1024 \
  -cores 2
  