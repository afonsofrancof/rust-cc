O ficheiro de configuração deve ter uma sintaxe que segue o seguinte conjunto de regras, sendo capaz de distinguir as situações de incoerência de configuração resultantes de um ficheiro com sintaxe desconhecida, registando estes incidentes nos logs e terminando a execução.

- As linhas iniciadas por **'#'** são consideradas comentários e serão ignoradas
- As linhas em branco também são ignoradas
- Cada linha deve conter uma definição de um parâmetro de configuração, seguindo a sintaxe: 

>[!Sintaxe Correta]
> **'parâmetro' 'tipo do valor' 'valor associado ao parâmetro'**

Os valores que são aceites (todas as referências a domínios, quer nos parâmetros quer nos valores, são considerados nomes completos):

>[!info] Parâmetros Aceites
>
>>[!Example] DB
>>O valor indica o ficheiro da base de dados com a informação do domínio indicado no parâmetro (o servidor assume o papel de SP para este domínio);
>
>>[!Example] SP
>>O valor indica o endereço IP[:porta] do SP do domínio indicado no parâmetro (o servidor assume o papel de SS para este domínio);
>
>>[!Example] SS
>>O valor indica o endereço IP[:porta] dum SS do domínio indicado no parâmetro (o servidor assume o papel de SP para este domínio) e que passa a ter autorização para pedir a transmissão da informação da base de dados (transferência de zona); podem existir várias entradas para o mesmo parâmetro (uma por cada SS do domínio)
>
>>[!Example] DD
>>O valor indica o endereço IP[:porta] dum SR, dum SS ou dum SP do domínio por defeito indicado no parâmetro (o servidor assume o papel de SR); podem existir várias entradas para o mesmo parâmetro (uma por cada servidor do domínio por defeito
>
>>[!Example] ST
>>O valor indica o ficheiro com a lista dos ST (o parâmetro deve ser igual a “root”)
>
>>[!Example] LG
>>O valor indica o ficheiro de log que o servidor deve utilizar para registar a atividade do servidor associada ao domínio indicado no parâmetro; só podem ser indicados domínios para o qual o servidor é SP ou SS; tem de existir pelo menos uma entrada a referir um ficheiro de log para toda a atividade que não seja diretamente referente aos domínios especificados noutras entradas LG (neste caso o parâmetro deve ser igual a “all”)


Exemplo de um ficheio de configuração de um SP no domínio example.com:
```
# Configuration file for primary server for example.com example.com DB /var/dns/example-com.db
example.com SS 193.123.5.189
example.com SS 193.123.5.190:5353
example.com DD 127.0.0.1
example.com LG /var/dns/example-com.log
all LG /var/dns/all.log
root ST /var/dns/rootservers.db
```



