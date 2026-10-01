# trabalho_evento_08

*Extraído de: trabalho_evento_08.pdf*

---



## Página 1

Estrutura Organizacional Emergente
em Sistemas Multiagentes: Agentes Cooperando
como os Insetos Sociais
Paulo R. Ferreira Jr.* and Ana L. C. Bazzan
Instituto de Inform´atica – Universidade Federal do Rio Grande do Sul
Caixa Postal 15.064 – 91.501-970 – Porto Alegre – RS – Brazil
{prferreiraj,bazzan}@inf.ufrgs.br
*doutorando, conclus˜ao em setembro de 2007
Resumo Agentes atuando em comunidade devem agir de maneira coe-
rente para atingir um objetivo comum. A coordena¸c˜ao pode ser baseada
na estrutura organizacional, onde a comunidade de agentes atua a fa-
vor de um objetivo comum atrav´es da forma como est˜ao organizados.
Em ambientes dinˆamicos a organiza¸c˜ao dos agentes deve se adaptar a
mudan¸cas nos objetivos do sistema, na disponibilidade de recursos, nos
relacionamentos entre os agentes, e assim por diante. Esta ﬂexibilidade ´e
um problema chave nos sistemas multiagentes e est´a relacionada `a mode-
los de aprendizagem e adapta¸c˜ao como os observados nos insetos sociais.
O presente trabalho prop˜oe uma abordagem para a gera¸c˜ao e adapta¸c˜ao
da organiza¸c˜ao de um sistema multiagente de larga escala, em tempo
de execu¸c˜ao, utilizando como base os modelos te´oricos de organiza¸c˜ao
das colˆonias de insetos sociais. Pretende-se que este trabalho contribua
para o avan¸co do estado-da-arte em sistemas multiagentes uma vez que
a abordagem proposta comp˜oe uma abordagem para a especiﬁca¸c˜ao e
resolu¸c˜ao de problemas, atrav´es destes sistemas, focada em ambientes
dinˆamicos e de larga escala.
Palavras-chave Organiza¸c˜ao em SAM, Comportamento Coletivo e Emer-
gente, Aloca¸c˜ao de Recursos e Tarefas em MAS.
1
Introdu¸c˜ao
O paradigma baseado nos sistemas multiagentes tem a coordena¸c˜ao como ponto
central. A coordena¸c˜ao ´e o processo no qual os agentes se engajam para garantir
que o conjunto deles que comp˜oe o sistema ir´a atuar de uma maneira coerente.
Quando agentes distribu´ıdos trabalham para cumprir determinado objetivo, eles
devem agir como uma unidade, coordenando suas a¸c˜oes, minimizando esfor¸cos
redundantes, compartilhando recursos, etc.
A principal quest˜ao abordada por este trabalho ´e como coordenar um grande
n´umero de agentes em um sistema multiagentes para atuar em ambientes dinˆa-
micos e de larga escala na busca de um objetivo comum.
A coordena¸c˜ao de um sistema multiagentes, atuando em ambientes com as
caracter´ısticas descritas acima, pode ser obtida atrav´es de sua estrutura organi-
zacional. A dimens˜ao funcional desta organiza¸c˜ao deﬁne os objetivos do sistema,


## Página 2

as tarefas necess´arias para atingir estes objetivos, como estas tarefas se dividem
em sub-tarefas e como estas se relacionam entre si. A dimens˜ao estrutural desta
organiza¸c˜ao deﬁne que agentes atuam em quais tarefas e em que tempo isto deve
ocorrer.
Desta forma, a dimens˜ao funcional descreve o ambiente em que o sistema
est´a inserido e quais os passos necess´arios para que os objetivos deste sistema
sejam atingidos. Tal dimens˜ao pode ser especiﬁcada utilizando uma linguagem
independente de dom´ınio como o TÆMS [4]. A principal diﬁculdade com rela¸c˜ao
ao problema em quest˜ao est´a na deﬁni¸c˜ao da dimens˜ao estrutural da organiza¸c˜ao.
Este trabalho parte do pr´e-suposto de que ´e poss´ıvel conseguir a emergˆen-
cia da dimens˜ao estrutural da organiza¸c˜ao atrav´es da utiliza¸c˜ao de modelos de
divis˜ao de trabalho em colˆonias de insetos sociais.
As colˆonias de insetos sociais tˆem uma das estrat´egias de sobrevivˆencia mais
bem sucedidas da natureza. Esta estrat´egia ´e centrada na organiza¸c˜ao das colˆo-
nias, cujas principais caracter´ısticas s˜ao a divis˜ao do trabalho e a especializa¸c˜ao.
As necessidades das colˆonias se modiﬁcam durante o passar do tempo. Estas
modiﬁca¸c˜oes est˜ao associadas com a fase de desenvolvimento da colˆonia, a ´epoca
do ano, a disponibilidade de alimentos, a press˜ao de predadores e as condi¸c˜oes
clim´aticas. A despeito dessas dr´asticas varia¸c˜oes nas condi¸c˜oes da colˆonia os
insetos sociais tˆem not´orio sucesso ecol´ogico.
Uma colˆonia de insetos sociais opera sem coordena¸c˜ao expl´ıcita. Um inseto
pertencente a casta oper´aria individualmente n˜ao conhece as necessidades da
colˆonia como um todo; este tem apenas informa¸c˜oes locais simples, capacidade
de comunica¸c˜ao extremamente limitada e ningu´em ´e encarregado de coordenar as
a¸c˜oes. Atrav´es da agrega¸c˜ao destes trabalhadores, o comportamento da colˆonia
emerge sem qualquer tipo de planejamento ou coordena¸c˜ao expl´ıcita. A chave
para este comportamento emergente est´a na plasticidade da divis˜ao de trabalho
nas colˆonias. Estas colˆonias respondem `a varia¸c˜ao nas condi¸c˜oes do ambiente
atrav´es do ajuste das taxas de trabalhadores engajados nas diversas tarefas que
precisam ser executadas.
Este trabalho prop˜oe uma abordagem para gerar e adaptar a dimens˜ao estru-
tural da organiza¸c˜ao dos sistemas multiagentes, atrav´es da aloca¸c˜ao e re-aloca¸c˜ao
de tarefas entre os agentes, utilizando como base os modelos te´oricos de divis˜ao
do trabalho dos insetos sociais.
O problema em quest˜ao pode ser redeﬁnido como um problema de aloca¸c˜ao /
escalonamento dinˆamico distribu´ıdo de tarefas em larga escala entre agentes com
diferentes competˆencias. Por aloca¸c˜ao entende-se o processo de atribuir agentes a
tarefas em tempo real, onde n˜ao ´e poss´ıvel despender tempo planejando, ou n˜ao
se tem informa¸c˜ao suﬁciente, e os agentes devem aprender com as experiˆencias
passadas. Por escalonamento entende-se o processo de determinar que agentes
realizam quais tarefas e organizar a execu¸c˜ao das tarefas com rela¸c˜ao ao tempo.
Neste ´ultimo, os agentes disp˜oe de tempo e informa¸c˜ao suﬁcientes para planejar
a aloca¸c˜ao das tarefas e sua disposi¸c˜ao no tempo deve ser feita de maneira a
minimizar o tempo total necess´ario para a realiza¸c˜ao das mesmas.


## Página 3

Dois problemas da ´area de pesquisa operacional modelam partes diferen-
tes, por´em complementares, do problema descrito acima. O Generalized Assign-
ment Problem (GAP) [19] e o Resource-Constrained Project Scheduling Problem
(RCPSP) [13]. Ambos s˜ao NP-dif´ıceis. Estes problemas n˜ao s˜ao originalmente
dinˆamicos nem distribu´ıdos. O n´umero de tarefas e agentes (ou recursos) s˜ao
ﬁxos. Contudo, podem ser facilmente estendidos para lidar com essas caracter´ıs-
ticas.
O GAP busca maximizar o ganho ao atribuir n tarefas a m agentes, sendo
que cada tarefa ´e associada a um agente apenas de acordo com restri¸c˜oes de com-
petˆencia destes agentes. No RCPSP tarefas devem ser escalonadas considerando
um conjunto limitado de agentes (recursos na deﬁni¸c˜ao original) necess´arios para
a realiza¸c˜ao de cada tarefa. As tarefas s˜ao relacionadas por restri¸c˜oes de inter-
dependˆencia de execu¸c˜ao, ou seja, determinadas tarefas devem ser executadas
antes de outras. O objetivo ´e buscar um escalonamento de todas as tarefas no
menor tempo poss´ıvel.
Estes problemas comp˜oem dois cen´arios para a experimenta¸c˜ao da aborda-
gem proposta neste trabalho. Al´em de modelar v´arios aspectos de aplica¸c˜oes
reais, estes problemas foram largamente estudados e existem conjuntos de dados
dispon´ıveis que podem ser usados para compara¸c˜ao. Pretende-se ainda esten-
der os dois cen´arios, ou a combina¸c˜ao deles, para lidar com outros aspectos que
envolvem a rela¸c˜ao entre tarefas que n˜ao s˜ao levados em conta nestes problemas.
Dentre ass contribui¸c˜oes esperadas com este trabalho, destacam-se: A abor-
dagem proposta ´e a ´unica abordagem para coordena¸c˜ao de agentes para cen´arios
dinˆamicos e de larga escala que oferece simultaneamente os mecanismos necess´a-
rios para a aloca¸c˜ao (em tempo real)e o escalonamento (com planejamento) de
tarefas de forma distribu´ıda; Pretende-se resolver de forma aproximada o GAP
(e suas extens˜oes). O GAP vˆem sendo utilizado para modelar muitos problemas
de aloca¸c˜ao de tarefas, incluindo cen´arios de coordena¸c˜ao de equipes de resgate
e salvamento, onde um n´umero muito restrito de solu¸c˜oes distribu´ıdas foram
propostas; Pretende-se resolver de forma aproximada o RCPSP. Este problema
modela in´umeras aplica¸c˜oes reais, principalmente relacionadas ao escalonamento
de tarefas em manufaturas. Uma abordagem distribu´ıda, como a que est´a sendo
proposta, junta-se ao esfor¸co de outras solu¸c˜oes para tentar resolver este pro-
blema cr´ıtico no estilo moderno de produ¸c˜ao;
Este artigo est´a organizado da seguinte forma: Na se¸c˜ao 2 ser˜ao apresentados
os trabalhos relacionados envolvendo as ´areas de SMA e de Inteligˆencia Coletiva;
Na se¸c˜ao 3 ser˜ao discutidas as id´eias principais da abordagem proposta; Os
resultados preliminares obtidos, e outros que se pretende obter, ser˜ao mostrados
na se¸c˜ao 4; e, ﬁnalmente, na se¸c˜ao 5 ser˜ao apresentadas algumas considera¸c˜oes
acerca das primeiras conclus˜oes e as atividades que ser˜ao realizadas no futuro.
2
Trabalhos Relacionados
A adapta¸c˜ao da organiza¸c˜ao de um MAS, para melhorar sua eﬁciˆencia quando
o ambiente no qual este est´a inserido ´e dinˆamico, foi previamente proposta na


## Página 4

literatura. Essa adapta¸c˜ao foi originalmente denominada de auto-organiza¸c˜ao.
Os primeiros trabalhos em auto-organiza¸c˜ao de MAS enfocavam o balan¸co de
carga adaptativo [20] ou a re-aloca¸c˜ao de tarefas de acordo com a eﬁciˆencia geral
do sistema [21], sendo espec´ıﬁcos para determinados cen´arios ou oferecendo uma
vis˜ao global para os agentes. Em [10], os autores prop˜oem uma abordagem que
pretende ser mais gen´erica, baseada em TÆMS e composta de um m´etodo para
auto-diagn´ostico. Esta abordagem mostrou bons resultados no cen´ario experi-
mentado, mas segundo os pr´oprios autores algumas quest˜oes sobre sua eﬁciˆencia
em termos mais gerais continuam em aberto.
Recentemente, as pesquisas tem se voltado para um aspecto da organiza¸c˜ao
em sistemas multiagentes: a aloca¸c˜ao distribu´ıda de tarefas. O Distributed Cons-
traint Optimization Problem (DCOP) ´e um formalismo para modelar um grande
conjunto de problemas que envolvem a aloca¸c˜ao de tarefas e tˆem sido tema de
muitos avan¸cos nos ´ultimos anos [14,12,17]. Nestas abordagens, a aloca¸c˜ao de
tarefas pode ser realizada on-line permitindo que o sistema se adapte a dinami-
cidade do ambiente.
Uma quest˜ao importante tamb´em sobre as abordagens para a solu¸c˜ao de
DCOPs ´e se estas tˆem desempenho suﬁciente para serem aplicadas em problemas
reais. A grande maioria das abordagens para DCOP apresentam bons resultados
em cen´arios simples mas alguns trabalhos destacam sua ineﬁciˆencia em cen´arios
mais complexos [6]. Al´em disso, s˜ao limitados os experimentos encontrados na
literatura sobre o desempenho destes algoritmos em cen´arios dinˆamicos [11,15].
Neste sentido, [19] prop˜oem um algoritmo para DCOP especiﬁcamente para
ambientes dinˆamicos e em larga escala, denominado LA-DCOP (Low-communica-
tion Aproximation DCOP). Os autores destacam que este algoritmo superou to-
das as outras abordagens de aloca¸c˜ao distribu´ıda de tarefas usando um n´umero
menor de mensagens e obtendo um melhor desempenho em cen´arios de larga
escala. Os resultados da abordagem que est´a sendo proposta poder˜ao ser compa-
rados com os resultados obtidos pelo LA-DCOP. Por´em, o LA-DCOP apresenta
v´arias limita¸c˜oes em rela¸c˜ao a abordagem que est´a sendo proposta por ter sido
elaborado especiﬁcamente para resolver o GAP.
Os modelos te´oricos de divis˜ao de trabalho dos insetos sociais, que serviram
como base desse trabalho, tamb´em foram utilizados previamente na literatura
para a coordena¸c˜ao de agentes em ambientes dinˆamicos [2]. Neste artigo os auto-
res apresentam uma abordagem que ´e aplicada ao Dynamic Flow Shop Scheduling
Problem. [3] apresentam uma extens˜ao desta abordagem para o mesmo problema
de [2].Os resultados mostraram que a utiliza¸c˜ao dos modelos te´oricos dos insetos
sociais ´e competitivo, e em alguns casos at´e superior, a outros sistemas baseados
em agentes apresentados anteriormente. Contudo, n˜ao est´a claro se estas abor-
dagens s˜ao capazes de lidar com outros problemas de escalonamento e aloca¸c˜ao
de tarefas.
A maneira como as formigas forrageiam tamb´em inspirou a concep¸c˜ao de
abordagens para solucionar problemas de otimiza¸c˜ao, criando uma ´area de pes-
quisa conhecida como ACO (Ant Colony Optimization). As abordagens nessa
´area, apesar da mesma origem e de tratar problemas e cen´arios semelhantes,


## Página 5

n˜ao considera que as solu¸c˜oes precisam ser estabelecidas de maneira distribu´ıda
entre os agentes.
Muitas abordagens baseadas em ACO tˆem sido propostas para problemas
de escalonamento. Em [13] os autores apresentam uma abordagem para o Re-
source Constraint Project Scheduling Problem (RCPSP), conhecido com um
problema geral de escalonamento. Foram realizados experimentos comparando
esta abordagem com outras baseadas em diferentes tecnologias, onde a aborda-
gem baseada em ACO apresentou melhores resultados. O RCPSP ser´a utilizado
como um dos cen´arios para experimentar a abordagem que est´a sendo proposta
neste trabalho.
[22] prop˜oem a aplica¸c˜ao de ACO em CSPs (problemas relacionados aos
DCOPs, mencionados anteriormente nesta se¸c˜ao), apresentando um algoritmo
b´asico e discutindo as diversas abordagens para se obter a solu¸c˜ao do referido
problema usando este algoritmo. N˜ao se tem conhecimento ainda da diferen¸ca
de desempenho entre ACO e as outras solu¸c˜oes propostas para CSPs, bem como
n˜ao se sabe a diferen¸ca desta solu¸c˜ao especiﬁcamente se comparada a outras
tamb´em baseadas em ACO para problemas que possam ser representados como
CSPs.
3
Abordagem Proposta
O embasamento da abordagem proposta, como j´a mencionado, ´e a plasticidade
na divis˜ao do trabalho nas colˆonias de insetos sociais. Esta plasticidade emerge
da intera¸c˜ao entre os indiv´ıduos e deles com o ambiente, utilizando padr˜oes sim-
ples de comunica¸c˜ao e sem coordena¸c˜ao expl´ıcita. Centenas, e at´e milhares, de
indiv´ıduos comp˜oe uma colˆonia que se auto-organiza para atender suas necessi-
dades em um ambiente dinˆamico. A aloca¸c˜ao de tarefas entre os indiv´ıduos que
comp˜oem uma colˆonia deve ser tal que as varia¸c˜oes do ambiente sejam absorvidas
e que a energia gasta pela colˆonia seja a menor poss´ıvel.
Agentes podem decidir quais tarefas realizar e quando realiz´a-las inspira-
dos na forma como os insetos sociais tomam este tipo de decis˜ao, sem utilizar
processos expl´ıcitos de coordena¸c˜ao e buscando a maior eﬁciˆencia poss´ıvel. Este
comportamento ´e adequado para agentes que comp˜oem sistemas multiagentes
engajados em problemas dinˆamicos e em larga escala.
Para representar as tarefas necess´arias para que o SMA atinja seu obje-
tivo, al´em da especiﬁca¸c˜ao da organiza¸c˜ao hier´arquica destas tarefas e sua inter-
dependˆencia, a abordagem proposta utiliza a linguagem TÆMS [4]. Esta lingua-
gem permite especiﬁcar de uma forma bastante rica essa estrutura de tarefas. Os
agentes determinam de forma probabil´ıstica se devem se engajar na realiza¸c˜ao
de uma tarefa utilizando como base o modelo te´orico do limiar de resposta apre-
sentado em [1]. As tarefas, representadas em TÆMS, produzem est´ımulos para
os agentes de acordo com suas caracter´ısticas e relacionamentos.
Cada tarefa em TÆMS ´e dividida em sub-tarefas de maneira recursiva, com-
pondo uma estrutura em forma de ´arvore, a partir da TG (nodo raiz) at´e sua
forma indivis´ıvel, os m´etodos (nodos folha). Somente os m´etodos s˜ao de fato


## Página 6

executados pelos agentes. Cada m´etodo j possui qualidade qj, custo cj e du-
ra¸c˜ao dj como atributos. As sub-tarefas de uma mesma tarefa pai na ´arvore s˜ao
relacionadas por fun¸c˜oes denominadas Quality Accumulation Functions (QAFs).
A tendˆencia do agente i se engajar no m´etodo j ´e determinada pela rela¸c˜ao
entre o est´ımulo do m´etodo, o limiar interno do agente e a demanda produzida
pelas QAFs relacionadas a tarefa j, como mostra a equa¸c˜ao 1.
Tθij(sj) = σ
s2
j
s2
j + θ2
ij
+ (1 −σ)X(j)
(1)
onde:
sj est´ımulo associado com a tarefa j
θij limiar interno do agente i em rela¸c˜ao a tarefa j
X(j) demanda associada com as QAFs relacionadas ao m´etodo j, 0 ≤X(j) ≤1
σ constante para determinar o peso de X(j) no c´alculo da tendˆencia
Deve-se considerar que os agentes atuam em ambientes dinˆamicos, onde a
estrutura de tarefas descrita em TÆMS pode ser modiﬁcada em tempo de exe-
cu¸c˜ao: m´etodos podem aparecer e desaparecer; o n´umero de agentes pode mudar;
e podem variar os relacionamentos entre os m´etodos.
Pela quest˜ao de limite de espa¸co, os detalhes dos elementos que comp˜oe a
equa¸c˜ao acima n˜ao ser˜ao discutidos. Pelo mesmo motivo, os outros aspectos da
abordagem que ser˜ao apresentados a seguir n˜ao ser˜ao formalizados neste artigo.
A abordagem proposta, formalizada e discutida em datalhes, pode ser encontrada
em [5].
Em [1], os autores mostram que o sucess˜ao emergente de tarefas pode ser
obtida com o uso de limiares ﬁxos mas com aplicabilidade limitada. A especia-
liza¸c˜ao dos indiv´ıduos em conjuntos de tarefas ´e um aspecto do comportamento
das colˆonias de insetos sociais que ´e importante e bastante estudado [18].
A especializa¸c˜ao dos indiv´ıduos de acordo com sua idade ´e denominado pelos
bi´ologos de polietismo temporal. Nas abelhas esta ´e a principal forma de se obter
a divis˜ao de trabalho. Trabalhadores jovens realizam tarefas dentro da colm´eia
e trabalhadores mais velhos realizam tarefas fora dela, como defender a colˆonia
e forragear.
Prop˜oe-se uma modiﬁca¸c˜ao no modelo de especializa¸c˜ao dos insetos sociais,
apresentado em [1]. Nesta modiﬁca¸c˜ao, o limiar interno dos agentes ´e atualizado
de acordo com sua idade e a idade das tarefas. Por idade de um agente e de um
m´etodo entende-se o tempo em que os agentes est˜ao atuando no sistema e o
tempo em que o m´etodo est´a dispon´ıvel para execu¸c˜ao, respectivamente.
Atrav´es desta modiﬁca¸c˜ao, os agentes tˆem limiares de resposta altos para
m´etodos que est˜ao aguardando a mais tempo para serem executados e baixos
para m´etodos novos. Isso garante que as tarefas n˜ao permane¸cam muito tempo
aguardando para serem realizadas. Tarefas que foram percebidas a mais tempo
pelo sistema estimulam mais intensamente os agentes, pois estes ter˜ao um limiar
de resposta maior para estas tarefas. Al´em disso, agentes mais novos tem limia-
res mais baixos que agentes mais velhos com rela¸c˜ao aos mesmos m´etodos. A


## Página 7

id´eia por traz disso ´e especializar agentes mais velhos em um conjunto maior de
m´etodos em rela¸c˜ao aos agentes mais novos, como ocorre com os insetos sociais.
A especializa¸c˜ao dos indiv´ıduos de acordo com sua forma ´e denominada pelos
bi´ologos de polietismo morfol´ogico, ou simplesmente polimorﬁsmo. Nas colˆonias
de formigas o polimorﬁsmo tem um papel de suma importˆancia para a divis˜ao
do trabalho. O polimorﬁsmo pode ser modelado atrav´es do limiar de interno.
Para diferenciar os agentes e obter a especializa¸c˜ao morfol´ogica basta iniciar os
limiares internos de cada agente de acordo com as tarefas as quais sua “forma”´e
mais adaptada. Agentes que partem com limiares altos para determinadas tarefas
ter˜ao maior tendˆencia a realiz´a-las.
Uma vez que no problema que est´a sendo considerado neste trabalho os agen-
tes tem diferentes competˆencias para realizar diferentes tarefas, o limiar interno
do agente ser´a iniciado para reﬂetir esta competˆencia. Esta inicializa¸c˜ao s´o po-
der´a ocorrer para as tarefas previamente conhecidas pelos agentes. Caso isso n˜ao
ocorra, os agentes s˜ao iniciados com um limiar de n´ıvel intermedi´ario para tais
tarefas.
Como mencionado anteriormente, potencialmente cada inseto na colˆonia pode
realizar todos os tipos de tarefas, mesmo quando sua morfologia ´e especialmente
adaptada para alguma tarefa em espec´ıﬁco e independentemente de sua idade.
Assim, a despeito do polietismo temporal e do polimorﬁsmo, os indiv´ıduos de
uma colˆonia de insetos sociais podem se engajar em tarefas que n˜ao est˜ao asso-
ciadas a sua forma ou idade para atender `a demandas de acordo com as neces-
sidades da colˆonia. Se o n´umero de indiv´ıduos engajados em uma tarefa n˜ao ´e
suﬁciente ou se esses indiv´ıduos n˜ao est˜ao atendendo tal demanda a contento,
outros indiv´ıduos podem se engajar nesta tarefa.
Prop˜oe-se condicionar a atualiza¸c˜ao dos limiares internos ao sucesso do agente
em desempenhar uma tarefa. Assim, quanto mais eﬁciente ´e o agente na reali-
za¸c˜ao de uma tarefa, de acordo com sua competˆencia, mais especializado nesta
tarefa o agente ser´a. A inﬂuˆencia do polietismo temporal nesta especializa¸c˜ao
´e importante para garantir que o sistema n˜ao gere agentes super-especializados
que n˜ao atendam a novas tarefas.
4
Resultados Preliminares
Uma vers˜ao preliminar da abordagem em quest˜ao foi previamente proposta em
[16,9]. Nesta vers˜ao, algumas id´eias b´asicas da abordagem foram consideradas
mas de forma ainda muito limitada. Alguns resultados obtidos com essa vers˜ao
preliminar em um cen´ario de escalonamento de produ¸c˜ao, mais especiﬁcamente
o Machine Sequencing Problem, foram publicados em [7].
A abordagem proposta j´a est´a sendo experimentada em uma vers˜ao estendida
do GAP (denominada E-GAP), onde os primeiros resultados foram publicados
em [8]. Neste artigo foi proposto o Swarm-GAP, algoritmo que trata o E-GAP
de maneira aproximada e distribu´ıda. Pretende-se que o Swarm-GAP seja um
algoritmo simples e eﬁciente.


## Página 8

Os resultados dos experimentos realizados mostraram que a decis˜ao proba-
bil´ıstica, baseada nos modelos te´oricos j´a mencionados, permite aos agentes to-
marem suas decis˜oes de maneira razoavelmente coordenada. O Swarm-GAP teve
um bom desempenho, na m´edia, 20% pior que o obtido atrav´es de uma aborda-
gem gulosa centralizada. A ﬁgura 1 mostra os resultados do Swarm-GAP com-
parados com uma abordagem gulosa, onde a qualidade da solu¸c˜ao ´e determinada
pela recompensa obtida (reward) e varia de acordo com a quantidade de agentes
envolvidos (number of agents) na aloca¸c˜ao de 2000 tarefas.
Pode-se comparar, ainda que de forma superﬁcial, o desempenho do Swarm-
GAP com rela¸c˜ao ao LA-DCOP. Os experimentos realizados utilizaram o mesmo
cen´ario, com os mesmos parˆametros de conﬁgura¸c˜ao. Um gr´aﬁco semelhante ao
da ﬁgura 1 por´em com os resultados do LA-DCOP, inclusive na mesma escala,
pode ser encontrado em [19].
Atualmente est˜ao sendo realizados experimentos neste cen´ario para determi-
nar as caracter´ısticas da organiza¸c˜ao emergente, analisando o comportamento
dos agentes quanto a sua especializa¸c˜ao. Concomitantemente, est˜ao sendo im-
plementados em uma ambiente de simula¸c˜ao mais completo (Robocup Rescue)
o Swarm-GAP e o LA-DCOP para que uma compara¸c˜ao mais detalhada entre
o desempenho dos dois algoritmos possa ser realizada. Pretende-se ainda expe-
rimentar falhas na simula¸c˜ao no que tange ao canais de comunica¸c˜ao entre os
agentes, a percep¸c˜ao errada das tarefas e o conhecimento incompleto do cen´ario.
Figura 1. Compara¸c˜ao da qualidade do Swarm-GAP com rela¸c˜ao a uma estrat´egia
gulosa centralizada
5
Conclus˜oes Preliminares e Atividades Futuras
No problema mencionado na se¸c˜ao anterior, os agentes se adaptaram bem para
completar as tarefas, conﬁrmando a id´eia de que os sistemas multiagentes podem
se auto-organizar como os insetos sociais, obtendo o mesmo sucesso que estes
quando atuando cooperativamente para a resolu¸c˜ao de problemas distribu´ıdos.


## Página 9

Os problemas de pesquisa operacional mencionados est˜ao servindo como
cen´arios para a experimenta¸c˜ao da abordagem. Tais problemas se complemen-
tam para cobrir o escopo de cen´arios em que a abordagem proposta pode atuar,
permitindo que seu desempenho seja analisado detalhadamente e comparado
com outras abordagens. Pretende-se mostrar que o paradigma dos insetos so-
ciais pode ser utilizado para coordenar sistemas multiagentes quando atuando
em ambientes dinˆamicos e de larga escala.
Pretende-se ainda realizar as seguintes atividades: Comparar diretamente os
resultados obtidos no cen´ario de E-GAP com os resultados apresentados em
[19], como mencionado na se¸c˜ao anterior; Experimentar a abordagem proposta
no cen´ario de RCPSP, analisando e comparando os resultados obtidos; Experi-
mentar a abordagem proposta em uma combina¸c˜ao dos cen´arios anteriores para
valid´a-la; e Estudar as caracter´ısticas das organiza¸c˜oes de agentes obtidas nos
cen´arios experimentados.
Referˆencias
1. E. Bonabeau, M. Dorigo, and G. Theraulaz.
Inspiration for optimization from
social insect behaviour. Nature, pages 39–42, July 2000.
2. M. Campos, E. Bonabeau, G. Thraulaz, and J. Deneubourg. Dynamic scheduling
and division of labor in social insects. In Adaptive Behavior, volume 8–2, pages
83–96, 2001.
3. V. Cicirello and S. Smith. Improved routing wasps for distributed factory control.
Journal of Autonomous Agents and Multi-Agent Systems, 8(3):237–266, May 2004.
4. K. S. Decker and V. R. Lesser. Quantitative modeling of complex computational
task environments. In Proceedings of the 12th International Workshop on Distri-
buted Artiﬁcial Intelligence, pages 67–82, Hidden Valley, Pennsylvania, 1993.
5. P.
R.
Ferreira
Jr.
Estrutura
organizacional
emergente
em
sis-
temas
multiagentes.
Technical
report,
Universidade
Federal
do
Rio
Grande
do
Sul
-
UFRGS,
Janeiro
2006.
Proposta
de
Tese
(http://www.inf.ufrgs.br/maslab/pergamus/pubs/propostaFerreira.pdf.zip).
6. P. R. Ferreira Jr. and A. L. C. Bazzan.
Distributed meeting schedule through
cooperative mediation: analysing optapo’s performance in a complex scenario. In
A. Meisels, editor, Proceedings of the Sixth International Workshop on Distribu-
ted Constraint Reasoning (DCR2005) - Nineteenth International Conference on
Artiﬁcial Intelligence (IJCAI2005), pages 101–113, July 2005.
7. P. R. Ferreira Jr. and A. L. C. Bazzan. Self-organization of agents to solve machine
sequencing problems. In L. Panait, S. Sen, and E. Alonso, editors, Workshop on
Adaptation and Learning in Autonomous Agents and Multiagent Systems (ALAA-
MAS 2006) - 5th International Joint Conference on Autonomous Agents And Mul-
tiagent Systems (AAMAS 2006), pages 70–75, May 2006.
8. P. R. Ferreira Jr. and A. L. C. Bazzan. Swarm-gap: A swarm based approximation
algorithm for E-GAP. In N. R. Jennings, M. Tambe, T. Ishida, and S. D. Ram-
churn, editors, First International Workshop on Agent Technology for Disaster
Management (ATDM 2006) - 5th International Joint Conference on Autonomous
Agents And Multiagent Systems (AAMAS 2006), pages 49–55, May 2006.
9. P. R. Ferreira Jr., D. Oliveira, and A. C. Bazzan. A swarm based approach to adapt
the structural dimension of agents’ organizations. Journal of Brazilian Computer
Society - JBCS - Special Issue on Agents Organizations, 11(1):63–73, July 2005.


## Página 10

10. B. Horling, B. Benyo, and V. Lesser. Using self-diagnosis to adapt organizational
structures.
In Proceedings of the 5th International Conference on Autonomous
Agents, pages 529–536, Montreal, June 2001. New York, ACM Press.
11. R. T. Maheswaran, M. Tambe, E. Bowring, J. P. Pearce, and P. Varakantham.
Taking DCOP to the real world: Eﬃcient complete solutions for distributed multi-
event scheduling. In Third International Joint Conference on Autonomous Agents
and Multiagent Systems, volume 1, pages 310–317, Washington, DC, USA, July
2004. IEEE Computer Society.
12. R. Mailler and V. Lesser. Solving distributed constraint optimization problems
using cooperative mediation. In Proceedings of the International Joint Conference
on Autonomous Agents and Multiagent Systems, 3., pages 438–445, New York,
2004. New York, IEEE Computer Society.
13. D. Merkle, M. Middendorf, and H. Schmeck. Ant colony optimization for resource-
constrained project scheduling. IEEE Transactions on Evolutionary Computation,
6(4):333–146, 2002.
14. P. J. Modi, W.-M. Shen, M. Tambe, and M. Yokoo. An asynchronous complete
method for distributed constraint optimization. In Second international joint con-
ference on Autonomous agents and multiagent systems, pages 161–168, New York,
NY, USA, 2003. ACM Press.
15. D. Oliveira, A. L. C. Bazzan, and V. Lesser.
Using cooperative mediation to
coordinate traﬃc lights: a case study. In Proceedings of the 4th International Joint
Conference on Autonomous Agents and Multi Agent Systems (AAMAS), pages
463–470. New York, IEEE Computer Society, July 2005.
16. D. Oliveira, P. R. Ferreira Jr., and A. L. C. Bazzan. A swarm based approach
for task allocation in dynamic agents organizations.
In Proceedings of the 3rd
International Joint Conference on Autonomous Agents and Multi Agent Systems,
AAMAS, volume 3, pages 1252–1253, New York, USA, July 2004. New York, IEEE
Computer Society.
17. A. Petcu and B. Faltings. A scalable method for multiagent constraint optimiza-
tion. In Nineteenth International Conference on Artiﬁcial Intelligence, Edinburgh,
Scotland, Aug 2005.
18. G. E. Robison. Regulation of division of labor in insect societies. Annual Review
of Entomology, 37:637–665, 1992.
19. P. Scerri, A. Farinelli, S. Okamoto, and M. Tambe. Allocating tasks in extreme
teams. In Proceedings of the fourth international joint conference on Autonomous
agents and multiagent systems, pages 727–734, New York, NY, USA, 2005. ACM
Press.
20. O. Shehory, K. Sycara, P. Chalasani, and S. Jha. Agent cloning: an approach to
agent mobility and resource allocation. IEEE Communications Magazine, 36(7):58–
67, 1998.
21. Y. So and E. Durfee. An organizational self-design model for organizational change.
In Workshop on AI and Theories of Groups and Organizations: Conceptual and
Empirical Research, pages 8–15, july 1993.
22. F. Tarrant and D. Bridge. When ants attack: Comparing ant algorithms for cons-
traint problems. In L. McGinty and B. Crean, editors, Procs. of the Fifteenth Irish
Conference on Artiﬁcial Intelligence and Cognitive Science, pages 167–176, 2004.
