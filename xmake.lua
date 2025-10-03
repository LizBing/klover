target("klover-c-modules")
    set_kind("static")
    add_includedirs("src", {public = true})
    add_files("src/memory/*.c")
