(function($){
	"use strict";

	// script for video widget
	$(document).ready(function(){
		$('body').on('click', '.gdlr-core-widget-upload-button', function(){
			var thumbnail = $(this).siblings('.gdlr-core-widget-upload-thumbnail');
			var img_id = $(this).siblings('.gdlr-core-widget-upload-id');
			var img_url = $(this).siblings('.gdlr-core-widget-upload-url');

			var frame = wp.media({
				title: "Upload",
				button: { text: "Select" },
				multiple: false
			}).on('select', function(){
	  
				// Get media attachment details from the frame state
				var attachment = frame.state().get('selection').first().toJSON();

				thumbnail.attr('src', attachment.url);
				img_id.val(attachment.id);
				img_url.val(attachment.url);
			}).open();
		});

		$('body').on('change', '.gdlr-core-widget-upload-url', function(){
			if( $(this).val() == '' ){
				$(this).siblings('.gdlr-core-widget-upload-thumbnail').attr('src', '');
				$(this).siblings('.gdlr-core-widget-upload-id').val('');
			}
		});
	});

	// sidebar generator
	
	function gdlr_core_sidebar_ajax( options ){
		var settings = $.extend({
			error: function(jqXHR, textStatus, errorThrown){
				gdlr_core_alert_box({ status: 'failed', head: gdlr_core_sidebar_generator.error_head, message: gdlr_core_sidebar_generator.error_message });
				
				// for displaying the debug text
				console.log(jqXHR, textStatus, errorThrown);
			},
			success: function(data){
				// default to adding sidebar action
				if( data.status == 'success' ){
					location.reload();
				}else if( data.status == 'failed' ){
					gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
				}
			}
		}, options);
		
		$.ajax({
			type: 'POST',
			url: gdlr_core_sidebar_generator.ajaxurl,
			data: settings.data,
			dataType: 'json',
			error: settings.error,
			success: settings.success
		});			
	}
	
	$(window).load(function(){

		var widget_right;
		var widget_form;
		
		function initWidget(){
			if( widget_right && widget_right.length ) return;

			widget_right = $('.block-editor-block-list__layout.is-root-container');
			widget_form = $('<form class="gdlr-core-sidebar-generator-wrapper">\
				<div class="gdlr-core-sidebar-generator-title">' + gdlr_core_sidebar_generator.title_text + '</div>\
				<input type="text" name="sidebar_name" />\
				<input type="hidden" name="security" value="' + gdlr_core_sidebar_generator.nonce + '" />\
				<input type="hidden" name="action" value="gdlr_core_add_sidebar" />\
				<input type="submit" value="' + gdlr_core_sidebar_generator.add_new_text + '" />\
			</form>').insertAfter(widget_right);

			// action to add new sidebar 
			widget_form.submit(function(){
				gdlr_core_sidebar_ajax({ data: $(this).serialize() });
				return false;
			});
		}
		
		var t = setInterval(function(){ 
			initWidget();

			var widget_area = widget_right.find('[data-widget-area-id]');

			if( widget_area.length ){

				// add remove button for dynamic sidebar
				widget_right.find('[data-widget-area-id]').each(function(){
					var widget_item = $(this).closest('.wp-block-widget-area').parent();
					var widget_id = $(this).attr('data-widget-area-id');
					var ignores = ['footer-1', 'footer-2', 'footer-3', 'footer-4', 'gdlr-core-sidebar-preset', 'wp_inactive_widgets'];
					if( ignores.indexOf(widget_id) >= 0 ){
						return true;
					}

					var remove_button = $('<div class="gdlr-core-delete-sidebar"></div>');
					remove_button.click(function(){
						gdlr_core_confirm_box({success: function(){
							gdlr_core_sidebar_ajax({ 
								data:{
									'action':'gdlr_core_remove_sidebar', 
									'security':gdlr_core_sidebar_generator.nonce, 
									'sidebar_id':widget_id
								},
								success: function(data){
									if( data.status == 'success' ){
										widget_item.slideUp(250, function(){
											$(this).remove();
											// wpWidgets.saveOrder();
										});
									}else if( data.status == 'failed' ){
										gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});	
									}
								}
							});	// gdlr-core-sidebar-ajax				
						}}); // confirm box
						
						return false;
					}); // remove button
					
					widget_item.find('.components-panel__body-title').append(remove_button);
				});

				clearTimeout(t);
			}

		}, 500);

		
		
			
	});

})(jQuery);	