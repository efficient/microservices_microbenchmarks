repeat_numlists() {
	local dirname="$1"
	tail -n1 "$dirname/repeat" | cut -d\" -f6
}

repeat_list() {
	local dirname="$1"
	local nlist="$2"
	tail -n1 "$dirname/repeat" | cut -d\" -f$((6 + 2 * nlist))
}

statistic() {
	local statidx="$1"
	local statname="$2"
	local filename="$3"

	local input=""
	if [ "$filename" = "-" ]
	then
		input="`cat`"
	fi
	tail -n"+`stats_first_line "$filename" <<-enil_tsrif_stats
		$input
	enil_tsrif_stats`" "$filename" <<-liat | grep "^$statname" | tail -n"+$statidx" | head -n1 | rev | cut -d" " -f1 | rev
		$input
	liat
}

unstats() {
	local filename="$1"

	local input=""
	if [ "$filename" = "-" ]
	then
		input="`cat`"
	fi
	head -n"`stats_first_line "$filename" <<-enil_tsrif_stats
		$input
	enil_tsrif_stats`" "$filename" <<-liat | tail -n+5 | head -n-2
		$input
	liat
}

stats_first_line() {
	local filename="$1"
	printf "%s\n" $((`grep -n "^$" "$filename" | tail -n+2 | head -n1 | cut -d: -f1` + 1))
}
