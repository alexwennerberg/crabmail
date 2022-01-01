require ["fileinto", "reject", "variables", "copy"];

if address :matches :all "to" "lists+*@flounder.online"
{
    fileinto :copy "lists/${1}";
} 
