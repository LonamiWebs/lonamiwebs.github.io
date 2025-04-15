+++
title = "Grafos"
date = 2017-06-02
updated = 2017-06-02
[taxonomies]
category = ["algos"]
tags = ["graphs"]
+++

<noscript>Hay cosas que no se van a ver a menos que actives JavaScript.
No *tracking*, ¡lo prometo!</noscript>

Imaginemos 5 estaciones de autobús, que denotaremos por ((s_i)):

<div class="matrix">
      ' s_1 ' s_2 ' s_3 ' s_4 ' s_5 \\
s_1   '     '  V  '     '     '     \\
s_2   '  V  '     '     '     '  V  \\
s_3   '     '     '     '  V  '     \\
s_4   '     '  V  '  V  '     '     \\
s_5   '  V  '     '     '  V  '
</div>

Esto se conoce como *"cuadro de interconexiones directas"*. Las ((V)) representan caminos conectados. Por ejemplo, en la primera fila partiendo de ((s_1)), llegando hasta la ((V)), se nos permite girar hacia arriba para llegar a ((s_2)).

Podemos ver esta misma tabla representada de una manera más gráfica:

![Tabla 1 como grafo](/blog/graphs/example1.svg)

Este tipo de gráfica es un grafo, y además dirigido (o *digrafo*), ya que el sentido en el que van las flechas sí importa. Está compuesto por vértices, unidos entre si por ejes (también llamados aristas o **arcos** dirigidos).

Se puede ir de un nodo otro mediante distintos **caminos** o *tours*. Por ejemplo, ((s_4 $rightarrow s_2 $rightarrow s_5)) es un camino indirecto de **orden** dos, porque debemos usar dos aristas para ir de ((s_4)) a ((s_5)).

Pasemos ahora a representar la matriz de **adyacencia** llamada A, que representa el mismo cuadro, pero usa ((1)) en vez de ((V)) para representar una conexión:

<div class="matrix">
  0 ' 1 ' 0 ' 0 ' 0 \\
  1 ' 0 ' 0 ' 0 ' 1 \\
  0 ' 0 ' 0 ' 1 ' 0 \\
  0 ' 1 ' 1 ' 0 ' 0 \\
  1 ' 0 ' 0 ' 1 ' 0
</div>

Así podemos ver como el elemento ((a_{2,1})) representa la conexión ((s_2 $rightarrow s_1)), y el ((a_{5,1})) la ((s_5 $rightarrow s_1)), etc.

En general, ((a_ij)) representa una conexión de ((s_i $rightarrow s_j)) siempre que ((a_{i,j} $geq 1)).

Trabajar con matrices nos permite tener una representación computable de un grafo cualquiera, lo cual es realmente útil.

<hr />

Los grafos tienen muchas más propiedades interesantes a parte de ser representables computacionalmente. ¿Qué ocurre si, por ejemplo, hallamos ((A^2))? Resulta la siguiente matriz:

<div class="matrix">
1 ' 0 ' 0 ' 0 ' 1 \\
1 ' 1 ' 0 ' 1 ' 0 \\
0 ' 1 ' 1 ' 0 ' 0 \\
1 ' 0 ' 0 ' 1 ' 1 \\
0 ' 2 ' 1 ' 0 ' 0
</div>

Podemos interpretar esta matriz como los caminos de orden **dos**. ¿Pero qué representa el elemento ((a_{5,2}=2))? Es el número de posibles caminos para ir de ((s_5 $rightarrow s_i $rightarrow s_2))

Es posible realizar la multiplicación de la fila y columna implicadas para ver qué elemento es el que hay que atravesar, así se tiene la fila (([1, 0, 0, 1, 0])) y la columna (([1, 0, 0, 1, 0])) (en vertical). Los elementos ((s_i·$geq 1)) son ((s_1)) y ((s_4)). Es decir, se puede ir de ((s_5)) a ((s_2)) o bien mediante ((s_5 $rightarrow s_1 $rightarrow s_2)) ó bien ((s_5 $rightarrow s_4 $rightarrow s_2)):

![Tabla anterior como grafo](/blog/graphs/example2.svg)

Es importante notar que en los gráfos no se consideran lazos, es decir, ((s_i $rightarrow s_i)) no está permitido; ni tampoco se trabaja con multigrafos (que permiten muchas conexiones, por ejemplo, de un número arbitrario ((n)) de veces.

Terminemos con ((A^3)):

<div class="matrix">
1 ' 1 '  0  ' 1 ' 0 \\
1 ' 2 ' #1# ' 0 ' 1 \\
1 ' 0 '  0  ' 1 ' 1 \\
1 ' 2 '  1  ' 1 ' 0 \\
2 ' 0 '  0  ' 1 ' 2
</div>

Podemos ver como ha aparecido el primer ((1)) en ((a_{2,3})), lo que representa que el camino más corto es de al menos de orden tres.

<hr />

Un grafo es **fuertemente conexo** siempre que se pueda encontrar una conexión para *todos* los elementos.

Para ver todos los caminos posibles hasta ahora, basta con sumar las formas directas más las formas indirectas, por lo que hasta ahora podemos sumar ((A + A^2 + A^3)) tal que:

<div class="matrix">
2 ' 2 ' 0 ' 1 ' 1 \\
3 ' 3 ' 1 ' 1 ' 3 \\
1 ' 1 ' 1 ' 2 ' 1 \\
2 ' 3 ' 2 ' 2 ' 1 \\
3 ' 2 ' 1 ' 2 ' 2
</div>

Sigue sin haber una conexión entre ((s_1)) y ((s_3)). Calculando ((A^4)):

<div class="matrix">
1 ' 2 ' 1 '   '   \\
  '   '   '   '   \\
  '   '   '   '   \\
  '   '   '   '   \\
  '   '   '   '
</div>

No hace falta seguir calculando, ya tenemos un grafo totalmente conexo.

<hr />

¡Felicidades! Has completado esta pequeña introducción a los gráficos. Ahora puedes jugar tú y diseñar tus propias conexiones.

Mantén pulsado el botón izquierdo del ratón en el área de arriba y arrastra hacia abajo para crear un nuevo nodo, o arrastra un nodo a este área para eliminarlo.

Para crear nuevas conexiones, mantén pulsado el botón derecho del ratón en el nodo del que quiera partir, y arrástralo hasta el nodo con el que lo quieras conectar.

Para eliminar las conexiones que salen de un nodo en concreto, haz clic con el botón central del ratón en el nodo que quieras.

<table><tr><td style="width:100%;">
  <button onclick="resetConnections()">Reiniciar conexiones</button>
  <button onclick="clearNodes()">Limpiar todos los nodos</button>
  <br />
  <br />
  <label for="matrixOrder">Mostrar matriz de orden:</label>
  <input id="matrixOrder" type="number" min="1" max="5"
                          value="1" oninput="updateOrder()">
  <br />
  <label for="matrixAccum">Mostrar matriz acumulada</label>
  <input id="matrixAccum" type="checkbox" onchange="updateOrder()">
  <br />
  <br />
  <div>
    <table id="matrixTable"></table>
  </div>
</td><td>
  <canvas id="canvas" width="400" height="400" oncontextmenu="return false;">
  Parece que tu navegador no vas a poder probar el ejemplo en tu navegador :(
  </canvas>
  <br />
</td></tr></table>

<script src="/blog/graphs/tinyparser.js"></script>
<script src="/blog/graphs/enhancements.js"></script>
<script src="/blog/graphs/graphs.js"></script>
