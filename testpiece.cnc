%
(teststukje)
(T99 D=10 CR=0 TAPER=10deg - ZMIN=0 - chamfer mill)
N10 G90 G94
N15 G17
N20 G21
N25 M9
N30 G28

(Proef climb)
N35 T99 M6
N40 S1 M3
N45 G54
N50 G0 X36.388 Y140.6
N55 G43 Z61.2
N60 G1 Z60 F1500
N65 G19 G2 Y80.6 Z0 J-60
N70 G1 Y76.6
N75 G17 G3 X40.388 Y72.6 I4
N80 G1 X55.188
N85 Y70.45
N90 X115.188
N95 Y72.6
N100 X129.988
N105 G2 X134.988 Y67.6 J-5
N110 G1 Y28
N115 G2 X129.988 Y23 I-5
N120 G1 X115.188
N125 Y25.15
N130 X55.188
N135 Y23
N140 X40.388
N145 G2 X35.388 Y28 J5
N150 G1 Y67.6
N155 G2 X40.388 Y72.6 I5
N160 G1 X50.388
N165 G0 Z61.2
N170 G28
N175 M30
%
