export DUMMY="${STORM_HOME}/target/dummy"
export SDCARD="${STORM_HOME}/target/sdcard"
export CAMERA_MODEL="MYCAMERA"
export PASSWORD="sw0rdf1sh"
export CLOUD="${STORM_HOME}/target/cloud"
export FICTION="Books/Fiction"
export SAVEGAMES="Games/Citra/Savegames"
export DB="${STORM_HOME}/target/db.txt"
export TASKER_TMP="${STORM_HOME}/target/tasker_tmp"
export STORM_BIN="${STORM_HOME}/scripts/run"
export DCIM_CAMERA="${SDCARD}/DCIM/Camera"
export PICTURES_CAMERA="${SDCARD}/Pictures/Camera"
export PICTURES_CAMERA_VV="${SDCARD}/Pictures/VV_Camera"
export STORM_CAMERA="Storm/phone/Pictures/Camera"
export DUMMY_IMG1="${DUMMY}/img1.jpg"
export DUMMY_IMG2="${DUMMY}/img2.jpg"
export GPS_LAT="10.50"
export GPS_LNG="-84.68"
export GPS_POS="10 deg 30' 0.00\" N, 84 deg 40' 48.00\" W"

apply_exif() {
   local -r path="$1"
   local -r android="${2:-false}"
   local -r check="${3:-false}"

   exiftool -overwrite_original_in_place -XMP:GPSLatitude="$GPS_LAT" -XMP:GPSLongitude="$GPS_LNG" -GPSLongitudeRef="West" -GPSLatitudeRef="North" "$path"

   if $android; then
      exiftool -config "${STORM_HOME}/scripts/mock/exiftool.conf" -overwrite_original_in_place -AndroidModel="$CAMERA_MODEL" "$path"
   else
      exiftool -overwrite_original_in_place -Model="$CAMERA_MODEL" "$path"
   fi

   if ! $check; then
      return
   fi

   local -r meta="$(exiftool "$path")"
   echo "$meta" | test::contains "$CAMERA_MODEL"
   echo "$meta" | test::contains "$GPS_POS"
}

create_dummy_imgs() {
   local -r path="$DUMMY_IMG1"
   local -r dir="$(dirname "$path")"
   local -r filename="$(basename "$path")"

   rm -rf "$path" || true
   mkdir -p "$dir" || true
   cd "$dir" || exit

   convert -size 320x240 -sampling-factor 1:1:1 -quality 100 -interlace JPEG -type Truecolor xc:blue "$filename"

   cp "$path" "$DUMMY_IMG2"

   apply_exif "$DUMMY_IMG1" false true
   apply_exif "$DUMMY_IMG2" true true
}

create_img() {
   local -r path="$1"
   local -r dir="$(dirname "$path")"

   mkdir -p "$dir" || true
   cd "$dir" || exit

   local dummy_path="$DUMMY_IMG1"
   local -r random=$(( ( RANDOM % 10 )  + 1 ))
   if [ $random -lt 5 ]; then
      dummy_path="$DUMMY_IMG2"
   fi

   cp "$dummy_path" "$path"
}

create_txt() {
   local -r path="$1"
   local -r dir="$(dirname "$path")"

   mkdir -p "$dir" || true
   cd "$dir" || exit
   echo "foo" > "$path"
}

cleanup() {
   rm -rf "$SDCARD" || true
   rm -rf "$CLOUD" || true
   rm -rf "$DUMMY" || true
}

get_folder_ids() {
   cd "$1"
   ls | grep -E "^[0-9A-Z]{4}$"
}

remove_empty_dirs() {
   for _ in $(seq 0 10); do
      find "$1" -type d -exec rmdir {} + 2>/dev/null || true
   done
}

setup_env() {
   export PATH="${STORM_HOME}/scripts/mock:${PATH}"

   local -r txt="$(cat <<EOF
=
Books
  Fiction
  Horror
Pictures
  Camera
=
2
spaceship.txt;44;201030
3
monster.txt;22;930606
5
beach and sand.jpg;123;211229
mountain.jpg;456;211230
EOF
   )"

   mkdir -p "$(dirname "$DB")"
   echo "$txt" > "$DB"

   create_dummy_imgs
}

run() {
   local -r yaml="$(cat <<EOF
crypto:
   password: ${PASSWORD}

cloud:
   providers:
      box:
          buffer: ${SDCARD}/Storm/box
          rclone: box
      vvdropbox:
          buffer: ${SDCARD}/Storm/vvdropbox
          rclone: vvdropbox
      pcloud:
          buffer: ${SDCARD}/Storm/pcloud
          rclone: pcloud
      alumni:
          buffer: ${SDCARD}/Storm/alumni
          rclone: alumni
      azure:
          buffer: ${SDCARD}/Storm/azure
          rclone: azure
      telegram:
          buffer: ${SDCARD}/Storm/telegram
      gphotos:
          buffer: ${SDCARD}/Storm/gphotos
          single_folder: true
          rclone: gphotos
          remote_path_fallback: album/rclone
      vvgphotos:
          buffer: ${SDCARD}/Storm/vvgphotos
          single_folder: true
          rclone: vvgphotos
          remote_path_fallback: album/rclone

camera:
   paths:
      - from: ${PICTURES_CAMERA}
        to: Pictures/Camera
        low_unzipped: gphotos
      - from: ${PICTURES_CAMERA_VV}
        to: VV/Pictures/Camera
        low_unzipped: vvgphotos

camera_backup:
   local_source: ${DCIM_CAMERA}
   local_intermediate: ${PICTURES_CAMERA}
   intermediate_provider: box
   intermediate_relative: ${STORM_CAMERA}
   ref_provider: gphotos

backup:
   provider: box
   max_kb: 1024
   denylist:
      - .*\.app
   paths:
      - from: ${SDCARD}/${FICTION}
        to: ${FICTION}
      - from: ${SDCARD}/Books/NonExisting
        to: Books/NonExisting
      - from: ${SDCARD}/${SAVEGAMES}
        to: ${SAVEGAMES}
      - from: ${SDCARD}/App/singlefile.txt
        to: Android/app_singlefile.txt

archive:
   max_file_kb: 100
   max_zip_kb: 420
   tmp_buffer: ${SDCARD}/Storm/azuretmp
   db_folder: ${SDCARD}/Storm/azuredb
   denylist:
   - .*deny.*

telegram:
   token: mockToken
   chat_id: mockChatId
   db_path: ${DB}
EOF
   )"

   mkdir -p "$TASKER_TMP" &>/dev/null || true
   bash "${STORM_HOME}/scripts/termux-run" "$(echo "$yaml" | tr '\"' '@')" "$@"
}

test::normalize_string() {
   echo "$*" | sed 's/[^[:print:]]//' | tr -cd '\11\12\15\40-\176' | tr -s ' ' | xargs
}

test::eq() {
   local -r actual="$(cat)"
   local -r expected="$1"

   local -r actual2="$(test::normalize_string "$actual")"
   local -r expected2="$(test::normalize_string "$expected")"

   if [[ "$actual2" != "$expected2" ]]; then
      echo "$actual"
      diff <(echo "$actual") <(echo "$expected") --color=always --suppress-common-lines --ignore-trailing-space --ignore-blank-lines
      exit 1
   fi
}

test::contains() {
   local -r haystack="$(cat)"
   local -r needle="$1"

   if [[ "$haystack" != *"$needle"* ]]; then
      echoerr "=== ${haystack} === doesn't contain === ${needle} ==="
      exit 1
   fi
}
