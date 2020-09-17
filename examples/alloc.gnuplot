set datafile separator ","
set terminal png size 1024, 768
set output 'alloc.png'
set ylabel "bytes"
set xlabel "size (u32)"
# set key autotitle columnhead
plot \
  'alloc.csv' using 1:2 title 'BTreeSet allocations' with lines, \
  '' using 1:3 title 'BTreeSet permanent' with lines, \
  '' using 1:5 title 'VecSet allocations' with lines, \
  '' using 1:6 title 'VecSet permanent' with lines, \
  '' using 1:8 title 'HashSet allocations' with lines, \
  '' using 1:9 title 'HashSet permanent' with lines, \
