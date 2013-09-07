rspt: *.rs nalgebra
	rustc -Llib -Lnalgebra/lib --opt-level=2 rspt.rs

nalgebra:
	git clone git://github.com/sebcrozet/nalgebra
	make -C nalgebra

