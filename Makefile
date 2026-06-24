all:
	cargo check --lib

gitaddall:
	git add src examples tests

loc:
	find src tests examples benches -name '*.rs' | xargs wc -l

loc_orig:
	find VTK -name '*.cpp' | xargs wc -l
	find VTK -name '*.h' | xargs wc -l
	find VTK -name '*.c' | xargs wc -l


gen-mods:
	@echo "No split-crate module generation is configured for the current monolithic src/ layout."

# Print project stats
stats:
	@echo "=== vtk-rs stats ==="
	@printf "Lines of Rust: "; find src examples tests benches -name '*.rs' | xargs cat | wc -l | tr -d ' '
	@printf "Image filters: "; find src/filters/image -maxdepth 1 -name '*.rs' 2>/dev/null | grep -v '/mod.rs' | wc -l | tr -d ' '
	@printf "Mesh filters:  "; find src/filters/mesh -maxdepth 1 -name '*.rs' 2>/dev/null | grep -v '/mod.rs' | wc -l | tr -d ' '
	@printf "Core filters:  "; find src/filters/core -maxdepth 1 -name '*.rs' 2>/dev/null | grep -v '/mod.rs' | wc -l | tr -d ' '
	@printf "Sources:       "; find src/filters/core/sources -maxdepth 1 -name '*.rs' 2>/dev/null | grep -v '/mod.rs' | wc -l | tr -d ' '
