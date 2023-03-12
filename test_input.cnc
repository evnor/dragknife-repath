(Test)
(T99 D=10 CR=0 TAPER=10deg - ZMIN=0 - chamfer mill)
N10 G90 G94 (absolute, feed per minute)
N15 G17 (XY plane)
N20 G21 (in mm)
N25 M9
N30 G28 (Go home)
N35 
(Snijden contour met scherpe hoeken)
N40 T99 M6
N45 G54 (Set coordinate offset)
N55 G43 Z51.2 (Tool length compensation)
N60 G1 X5 Y0 Z3
N60 G1 X10 Z0
N60 G1 X50 Z0
N65 G3 Y50 I-50 J0
N70 G1 X50 Y30
N75 G2 X48 Y28 I-2
N80 G3 X46 Y26 J-2