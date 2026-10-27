(function($){
	$(document).ready(function(){
		wp.hooks.addFilter('rank_math_content', 'goodlayers-core', function(content){
			$('#gdlr-core-page-builder').find('.gdlr-core-page-builder-item-container-preview').each(function(){
				content += $(this).children().not('script, style').html();
			});

			return content;
		});
	});
})(jQuery);