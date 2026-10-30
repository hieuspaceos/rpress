(function($){
	"use strict";
	
	var tourmasterThemeOption = function(){
		
		this.main_nav = $('#tourmaster-admin-option-nav');
		this.nav_slide = $('#tourmaster-admin-option-nav-slides');
		this.breadcrumbs = $('#tourmaster-admin-option-head-breadcrumbs');
		this.current_active_nav = this.main_nav.children('.tourmaster-active');
		
		this.sub_nav = $('#tourmaster-admin-option-subnav');
		this.content = $('#tourmaster-admin-option-body-content');
		
		this.search_btn = $('#tourmaster-admin-option-head-search-button');
		this.search_txt = $('#tourmaster-admin-option-head-search-text');
		this.save_btn = $('#tourmaster-admin-option-save-button');
		
		this.html_option = new tourmasterHtmlOption(this.content);

        this.page_slug = this.main_nav.attr('data-page');
		
		this.init();
	}
		
	tourmasterThemeOption.prototype = {
		
		init: function(){
			
			var t = this;
			
			// binding the main nav event
			t.mainNavBinding();
			
			// binding the sliding nav
			t.slideNavBinding();
			
			// binding the subnav event
			t.subNavBinding();
			
			// binding the save button
			t.saveBtnBinding();
			
			// binding the search button
			t.search_btn.click(function(){
				var keyword = t.search_txt.val();

				if( keyword.trim() == '' ){
					tourmaster_alert_box({ status: 'failed', head: $(this).attr('data-blank-keyword-head'), message: $(this).attr('data-blank-keyword') });
				}else{
					t.searchAction(keyword);
				}
			});
			t.search_txt.on('input', tourmaster_debounce(function(){
				var keyword = t.search_txt.val();
				
				if( keyword.trim() == '' ){
					t.current_active_nav.trigger('click');
				}else{
					t.searchAction(keyword);
				}
				
			}, 600));
		},
		
		
		// ajax action for theme option
		ajax_action: function( options ){
			
			var t = this;
			var settings = $.extend({
				beforeSend: function(){
					t.content.children('.tourmaster-active').animate({opacity: 0}, 150, function(){
						t.content.addClass('tourmaster-now-loading');
					});
				},
				error: function(jqXHR, textStatus, errorThrown){
					tourmaster_alert_box({ status: 'failed', head: tourmaster_ajax_message.error_head, message: tourmaster_ajax_message.error_message });
					
					// for displaying the debug text
					console.log(jqXHR, textStatus, errorThrown);
					
					t.content.removeClass('tourmaster-now-loading');
					t.content.children('.tourmaster-active').animate({opacity: 1}, 150);
				},
				success: function(data){
					// action from tab changing
					if( data.status == 'failed' ){
						tourmaster_alert_box({ status: data.status, head: data.head, message: data.message });
					}else if( data.status == 'success' ){
						t.breadcrumbs.html(data.breadcrumbs);
						t.sub_nav.html(data.subnav);
						t.content.html(data.content).removeClass('tourmaster-now-loading');
						t.content.children('.tourmaster-active').css({opacity: 0}).animate({opacity: 1}, 200);
						t.html_option.rebind();
							
						// bind content save button
						t.content.find('.tourmaster-admin-option-save-button').click(function(){
							t.save_btn.trigger('click');
						});
					}
				}
			}, options);
			
			$.ajax({
				type: 'POST',
				url: tourmaster_ajax_message.ajaxurl,
				data: settings.data,
				dataType: 'json',
				beforeSend: settings.beforeSend,
				error: settings.error,
				success: settings.success
			});			
			
		},
		
		// binding the click action on main navigation
		mainNavBinding: function(){
			var t = this;
			t.main_nav.children('.tourmaster-admin-option-nav-item').click(function(){
				if( $(this).hasClass('tourmaster-active') ) return;
				
				t.current_active_nav = $(this);
				t.current_active_nav.addClass('tourmaster-active').siblings().removeClass('tourmaster-active');
				t.current_active_nav.trigger('tourmaster-active-changed');
				
				t.ajax_action({
					data: { 
						'security': tourmaster_ajax_message.nonce, 
						'action': 'get_tourmaster_option_tab_' + t.page_slug, 
						'nav_order': $(this).attr('data-nav-order'), 
						'option': t.html_option.get_val() 
					}
				});
			});
		},
		
		// doing the search in theme option
		searchAction: function(keywords){
			var t = this;
			
			t.ajax_action({
				data: { 
					'security': tourmaster_ajax_message.nonce, 
					'action': 'get_tourmaster_option_search_' + t.page_slug, 
					'keyword': keywords, 
					'option': t.html_option.get_val() 
				},
				beforeSend: function(){
					t.main_nav.trigger('tourmaster-active-none');
					
					t.content.children('.tourmaster-active').animate({opacity: 0}, 150, function(){
						t.content.addClass('tourmaster-now-loading');
					});
				}
			});	
		},
		
		// binding the sliding nav when there're any action
		slideNavBinding: function(){
			var t = this;
			var active_nav = t.main_nav.children('.tourmaster-admin-option-nav-item.tourmaster-active');
			var orig_pos = {
				left: (active_nav.length > 0)? active_nav.position().left: 0,
				width: (active_nav.length > 0)? active_nav.width(): 0,
			}
			
			// init the nav position
			t.nav_slide.css({left: orig_pos.left, width: orig_pos.width});
			
			// hover action
			t.main_nav.children('.tourmaster-admin-option-nav-item').hover(function(){
				t.nav_slide.animate({left: $(this).position().left, width: $(this).width()}, { queue: false, easing: 'easeOutQuad', duration: 250 });
			}, function(){
				t.nav_slide.animate({left: orig_pos.left, width: orig_pos.width}, { queue: false, easing: 'easeOutQuad', duration: 250 });
			});
			
			// active position change 
			t.main_nav.children('.tourmaster-admin-option-nav-item').on('tourmaster-active-changed', function(){
				orig_pos.left = $(this).position().left;
				orig_pos.width = $(this).width();
				
				t.nav_slide.css({left: orig_pos.left, width: orig_pos.width});
			});
			
			// for search action
			t.main_nav.on('tourmaster-active-none', function(){
				orig_pos.left = 0;
				orig_pos.width = 0;
				
				t.nav_slide.css({left: orig_pos.left, width: orig_pos.width});
				t.current_active_nav.removeClass('tourmaster-active');
			});
		},
		
		// binding the event for subnav changing
		subNavBinding: function(){
			var t = this;
			t.sub_nav.on('click', '.tourmaster-admin-option-subnav-item', function(){
				if( $(this).hasClass('tourmaster-active') ) return;

				$(this).addClass('tourmaster-active').siblings('.tourmaster-active').removeClass('tourmaster-active');
				
				// normally trigger hide/show content
				var section = $(this).attr('data-subnav-slug');
				var section_text = $(this).html();
				
				t.content.children('.tourmaster-admin-option-section').each(function(){
					if( section == $(this).attr('data-section-slug') ){
						$(this).css({display: 'none'}).addClass('tourmaster-active').fadeIn(200);
						t.breadcrumbs.children('.tourmaster-admin-option-head-breadcrumbs-subnav').html(section_text).css('opacity', 0).animate({'opacity': 1}, 100);
					}else{
						$(this).removeClass('tourmaster-active').css({display: 'none'});
					}
				});

			});
		}, // subNavBinding
		
		// binding the save action on the button
		saveBtnBinding: function(){
			var t = this;
			
			t.save_btn.click(function(){
				
				t.save_btn.addClass('tourmaster-now-loading');
				t.content.find('.tourmaster-admin-option-save-button').addClass('tourmaster-now-loading');
				
				t.ajax_action({
					beforeSend: function(){
						t.save_btn.addClass('tourmaster-now-loading');
					},
					data: { 
						'security': tourmaster_ajax_message.nonce, 
						'action': 'save_tourmaster_option_' + t.page_slug, 
						'option': t.html_option.get_val() 
					},
					success: function(data){
						if( data.status == 'failed' ){
							tourmaster_alert_box({ status: data.status, head: data.head, message: data.message });
						}else if( data.status == 'success' ){
							tourmaster_alert_box({ status: data.status, head: data.head, message: data.message });
						}
						
						t.save_btn.removeClass('tourmaster-now-loading');
						t.content.find('.tourmaster-admin-option-save-button').removeClass('tourmaster-now-loading');
					}
				});
				
			});
			
			// bind the content button
			t.content.find('.tourmaster-admin-option-save-button').click(function(){
				t.save_btn.trigger('click');
			});
		}
		
	}
	
	$(document).ready(function(){
		
		new tourmasterThemeOption();
		
	});
	
})(jQuery);