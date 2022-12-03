use std::path::Path;

use crate::fs::IPathBuf;
use crate::shell::{self, ShellCmd, ShellError};

pub fn pictures(base_folder: &Path, folder_id: &str) -> Result<ShellCmd, ShellError> {
    let code = format!(
        r#"
_mv() {{
   local -r from="$1"
   local -r to="$2"

   if [ "$from" = "$to" ]; then
      return 0
   fi

   mkdir -p "$(dirname "$to")" || true
   mv "$from" "$to"
}}

move_pictures() {{
   local -r base_folder="{}"
   local -r folder_id="{}"

   IFS=$'\n'

   local -r files="$(find "$base_folder" -type f)"
   local -r arr=($files)

   local -r new_arr=($(echo "$files" \
      | sed -e "s|HEIC$|heic|" -e "s|heic$|heic.jpg|" \
      | sed -e "s|PNG$|png|" -e "s|png$|png.jpg|" \
      | sed -E "s|${{base_folder}}/([^/]+)\.|${{base_folder}}/${{folder_id}}/\1.|" \
      | sed -E 's|[^/a-zA-Z0-9.-]|_|g'))

   echo "${{new_arr[@]}}" >&2

   for (( i=0; i<${{#new_arr[@]}}; i++ )); do
     _mv "${{arr[$i]}}" "${{new_arr[$i]}}"
   done
}}

move_pictures
"#,
        base_folder.to_string(),
        folder_id,
    );

    println!("${code}");

    let args = &["-c", &code];
    shell::out("bash", args)
}
