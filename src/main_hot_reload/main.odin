// Development atlas exe. Loads atlas.dll and reloads it whenever it changes.

package main

import "core:dynlib"
import "core:fmt"
import "core:c/libc"
import "core:os"
import "core:log"
import "core:mem"

when ODIN_OS == .Windows {
	DLL_EXT :: ".dll"
} else when ODIN_OS == .Darwin {
	DLL_EXT :: ".dylib"
} else {
	DLL_EXT :: ".so"
}

// We copy the DLL because using it directly would lock it, which would prevent
// the compiler from writing to it.
copy_dll :: proc(to: string) -> bool {
	exit: i32
	when ODIN_OS == .Windows {
		exit = libc.system(fmt.ctprintf("copy atlas.dll {0}", to))
	} else {
		exit = libc.system(fmt.ctprintf("cp atlas" + DLL_EXT + " {0}", to))
	}

	if exit != 0 {
		fmt.printfln("Failed to copy atlas" + DLL_EXT + " to {0}", to)
		return false
	}

	return true
}

AtlasAPI :: struct {
	lib: dynlib.Library,
	init_window: proc(),
	init: proc(),
	update: proc() -> bool,
	shutdown: proc(),
	shutdown_window: proc(),
	memory: proc() -> rawptr,
	memory_size: proc() -> int,
	hot_reloaded: proc(mem: rawptr),
	force_reload: proc() -> bool,
	force_restart: proc() -> bool,
	modification_time: os.File_Time,
	api_version: int,
}

load_atlas_api :: proc(api_version: int) -> (api: AtlasAPI, ok: bool) {
	mod_time, mod_time_error := os.last_write_time_by_name("atlas" + DLL_EXT)
	if mod_time_error != os.ERROR_NONE {
		fmt.printfln(
			"Failed getting last write time of atlas" + DLL_EXT + ", error code: {1}",
			mod_time_error,
		)
		return
	}

	// NOTE: this needs to be a relative path for Linux to work.
	atlas_dll_name := fmt.tprintf("{0}atlas_{1}" + DLL_EXT, "./" when ODIN_OS != .Windows else "", api_version)
	copy_dll(atlas_dll_name) or_return

	// This proc matches the names of the fields in AtlasAPI to symbols in the
	// atlas DLL. It actually looks for symbols starting with `atlas_`, which is
	// why the argument `"atlas_"` is there.
	_, ok = dynlib.initialize_symbols(&api, atlas_dll_name, "atlas_", "lib")
	if !ok {
		fmt.printfln("Failed initializing symbols: {0}", dynlib.last_error())
	}

	api.api_version = api_version
	api.modification_time = mod_time
	ok = true

	return
}

unload_atlas_api :: proc(api: ^AtlasAPI) {
	if api.lib != nil {
		if !dynlib.unload_library(api.lib) {
			fmt.printfln("Failed unloading lib: {0}", dynlib.last_error())
		}
	}

	if os.remove(fmt.tprintf("atlas_{0}" + DLL_EXT, api.api_version)) != nil {
		fmt.printfln("Failed to remove atlas_{0}" + DLL_EXT + " copy", api.api_version)
	}
}

main :: proc() {
	context.logger = log.create_console_logger()

	default_allocator := context.allocator
	tracking_allocator: mem.Tracking_Allocator
	mem.tracking_allocator_init(&tracking_allocator, default_allocator)
	context.allocator = mem.tracking_allocator(&tracking_allocator)

	reset_tracking_allocator :: proc(a: ^mem.Tracking_Allocator) -> bool {
		err := false

		for _, value in a.allocation_map {
			fmt.printf("%v: Leaked %v bytes\n", value.location, value.size)
			err = true
		}

		mem.tracking_allocator_clear(a)
		return err
	}

	atlas_api_version := 0
	atlas_api, atlas_api_ok := load_atlas_api(atlas_api_version)

	if !atlas_api_ok {
		fmt.println("Failed to load Atlas API")
		return
	}

	atlas_api_version += 1
	atlas_api.init_window()
	atlas_api.init()

	old_atlas_apis := make([dynamic]AtlasAPI, default_allocator)

	window_open := true
	for window_open {
		window_open = atlas_api.update()
		force_reload := atlas_api.force_reload()
		force_restart := atlas_api.force_restart()
		reload := force_reload || force_restart
		atlas_dll_mod, atlas_dll_mod_err := os.last_write_time_by_name("atlas" + DLL_EXT)

		if atlas_dll_mod_err == os.ERROR_NONE && atlas_api.modification_time != atlas_dll_mod {
			reload = true
		}

		if reload {
			new_atlas_api, new_atlas_api_ok := load_atlas_api(atlas_api_version)

			if new_atlas_api_ok {
			force_restart = force_restart || atlas_api.memory_size() != new_atlas_api.memory_size()

				if !force_restart {
					// This does the normal hot reload

					// Note that we don't unload the old atlas APIs because that
					// would unload the DLL. The DLL can contain stored info
					// such as string literals. The old DLLs are only unloaded
					// on a full reset or on shutdown.
					append(&old_atlas_apis, atlas_api)
					atlas_memory := atlas_api.memory()
					atlas_api = new_atlas_api
					atlas_api.hot_reloaded(atlas_memory)
				} else {
					// This does a full reset. That's basically like opening and
					// closing the atlas, without having to restart the executable.
					//
					// You end up in here if the atlas requests a full reset OR
					// if the size of the atlas memory has changed. That would
					// probably lead to a crash anyways.

					atlas_api.shutdown()
					reset_tracking_allocator(&tracking_allocator)

					for &g in old_atlas_apis {
						unload_atlas_api(&g)
					}

					clear(&old_atlas_apis)
					unload_atlas_api(&atlas_api)
					atlas_api = new_atlas_api
					atlas_api.init()
				}

				atlas_api_version += 1
			}
		}

		if len(tracking_allocator.bad_free_array) > 0 {
			for b in tracking_allocator.bad_free_array {
				log.errorf("Bad free at: %v", b.location)
			}

			// This prevents the atlas from closing without you seeing the bad
			// frees. This is mostly needed because I use Sublime Text and my atlas's
			// console isn't hooked up into Sublime's console properly.
			libc.getchar()
			panic("Bad free detected")
		}

		free_all(context.temp_allocator)
	}

	free_all(context.temp_allocator)
	atlas_api.shutdown()
	if reset_tracking_allocator(&tracking_allocator) {
		// This prevents the atlas from closing without you seeing the memory
		// leaks. This is mostly needed because I use Sublime Text and my atlas's
		// console isn't hooked up into Sublime's console properly.
		libc.getchar()
	}

	for &g in old_atlas_apis {
		unload_atlas_api(&g)
	}

	delete(old_atlas_apis)

	atlas_api.shutdown_window()
	unload_atlas_api(&atlas_api)
	mem.tracking_allocator_destroy(&tracking_allocator)
}

// Make atlas use good GPU on laptops.

@(export)
NvOptimusEnablement: u32 = 1

@(export)
AmdPowerXpressRequestHighPerformance: i32 = 1
