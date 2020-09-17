set datafile separator ","
set terminal png size 1024, 768
set output 'nalloc.png'
set ylabel "bytes"
set xlabel "size (u64)"
# set key autotitle columnhead
plot \
  'alloc.csv' using 1:4 title 'BTreeSet allocations' with lines, \
  '' using 1:7 title 'VecSet allocations' with lines, \
  '' using 1:10 title 'HashSet allocations' with lines, \
